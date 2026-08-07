use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use crossbeam_channel::{Sender, unbounded};
use flutter_rust_bridge::frb;
use parking_lot::Mutex;

use crate::{
    api::{
        enums::{AudioCommand, MAX_RATE, MIN_RATE},
        filters::{
            effects::{ChorusEffect, DelayEffect, HardClipEffect, OverdriveEffect, ReverbEffect},
            equalizer::{BandConfig, Equalizer},
        },
        renderer::{
            output::{AudioOutput, AudioOutputConfig},
            state::{
                AudioState, BuffConfig, DspState, PlaybackState, SharedControlState,
                make_sample_queue,
            },
        },
        source::{
            AudioSource,
            decode::{DecodeCommand, DecodePool},
        },
        visualizer::{BandEnergy, VBandKind, VisualizerProcessor},
    },
    info, warn,
};

use crate::api::enums::{DEFAULT_MAX_QUEUE_SECONDS, DEFAULT_VISUALIZER_SECONDS};

#[frb(opaque)]
pub struct AudioPlayer {
    // ── Decoding ──────────────────────────────────────────────────────────────
    decode: DecodePool,

    // ── Atomic control state (no mutex for hot-path reads) ────────────────────
    ctrl: Arc<SharedControlState>,

    // ── Current queue producer — sent to the decode thread on start ───────────
    // Wrapped in Mutex so AudioPlayer implements Sync (required by frb).
    // ringbuf::HeapProd<f32> uses Cell<usize> internally which is !Sync.
    queue_prod: Mutex<Option<ringbuf::HeapProd<f32>>>,

    // ── AudioState: queue consumer + visualizer, guarded by Mutex ─────────────
    audio_state: Arc<Mutex<AudioState>>,

    // ── DSP: EQ + effects, behind its own Mutex ───────────────────────────────
    dsp: Arc<Mutex<DspState>>,

    // ── Audio output stream ───────────────────────────────────────────────────
    output: Arc<Mutex<AudioOutput>>,

    // ── Cached device config (read without locking output) ────────────────────
    output_config: AudioOutputConfig,

    // ── Decode command sender for the device-pool → seek path ─────────────────
    active_cmd_tx: Arc<Mutex<Option<Sender<DecodeCommand>>>>,

    // ── Visualizer ───────────────────────────────────────────────────────────
    visualizer_pc: Option<Mutex<VisualizerProcessor>>,
    visualizer_fft_size: usize,

    // ── Device-pool thread ────────────────────────────────────────────────────
    device_pool: Option<thread::JoinHandle<()>>,
    device_pool_stop: Arc<AtomicBool>,
}

// SAFETY: ringbuf::HeapProd<f32> contains Cell<usize> (non-Sync) but is wrapped in
// Mutex, so concurrent access is impossible.  AudioPlayer is always accessed
// through frb's serialised dispatch.
unsafe impl Sync for AudioPlayer {}

impl AudioPlayer {
    #[frb(sync)]
    pub fn new() -> AudioPlayer {
        let decode = DecodePool::new();

        let out_config = AudioOutput::get_output_config();
        let sample_rate = out_config.sample_rate;
        let channels = out_config.channels as usize;

        let max_samples = sample_rate as usize * channels * DEFAULT_MAX_QUEUE_SECONDS;
        let v_max = sample_rate as usize * channels * DEFAULT_VISUALIZER_SECONDS;

        // ── Atomic control state ──────────────────────────────────────────────
        let ctrl = SharedControlState::new();

        // ── Lock-free SPSC queue ──────────────────────────────────────────────
        let (queue_prod, queue_cons) = make_sample_queue(max_samples);

        // ── AudioState ────────────────────────────────────────────────────────
        let audio_state = Arc::new(Mutex::new(AudioState::new_with_ctrl(
            Arc::clone(&ctrl),
            queue_cons,
            BuffConfig::new(max_samples, DEFAULT_MAX_QUEUE_SECONDS, sample_rate),
            BuffConfig::new(v_max, DEFAULT_VISUALIZER_SECONDS, sample_rate),
            channels,
        )));

        // ── DspState ─────────────────────────────────────────────────────────
        let dsp = Arc::new(Mutex::new(DspState::new(sample_rate as f32)));

        // ── Device rebuild channel ────────────────────────────────────────────
        let (rebuild_tx, rebuild_rc) = unbounded::<AudioCommand>();

        // ── AudioOutput ───────────────────────────────────────────────────────
        let output = Arc::new(Mutex::new(AudioOutput::new(
            rebuild_tx,
            Arc::clone(&audio_state),
            Arc::clone(&dsp),
            out_config,
        )));

        // ── active_cmd_tx ─────────────────────────────────────────────────────
        let active_cmd_tx: Arc<Mutex<Option<Sender<DecodeCommand>>>> = Arc::new(Mutex::new(None));
        let active_cmd_tx_clone = Arc::clone(&active_cmd_tx);

        // ── Device-pool thread ────────────────────────────────────────────────
        let ctrl_dp = Arc::clone(&ctrl);
        let audio_state_dp = Arc::clone(&audio_state);
        let dsp_dp = Arc::clone(&dsp);
        let output_dp = Arc::clone(&output);
        let device_pool_stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&device_pool_stop);
        let snapshot_cfg = out_config;

        let device_pool = thread::Builder::new()
            .name("Device Pool".to_string())
            .spawn(move || {
                loop {
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }

                    match rebuild_rc.recv_timeout(std::time::Duration::from_millis(100)) {
                        Ok(AudioCommand::RebuildOuput) => {
                            let prev = ctrl_dp.state();
                            ctrl_dp.set_state(PlaybackState::Paused);

                            // Brief lock to dispose; build happens outside the lock.
                            {
                                output_dp.lock().dispose_old_stream();
                            }

                            let build_params = output_dp.lock().build_params();
                            let result = match build_params {
                                Some((tx, st, dsp)) => AudioOutput::build_stream(tx, st, dsp),
                                None => continue,
                            };

                            match result {
                                Ok((stream, new_cfg)) => {
                                    {
                                        let mut out = output_dp.lock();
                                        if out.has_stream() {
                                            continue; // another path installed while we built
                                        }
                                        out.install_stream(stream, new_cfg);
                                    }
                                    if new_cfg.sample_rate != snapshot_cfg.sample_rate
                                        || new_cfg.channels != snapshot_cfg.channels
                                    {
                                        let sr = new_cfg.sample_rate;
                                        let ch = new_cfg.channels as usize;
                                        let cap = sr as usize * ch * DEFAULT_MAX_QUEUE_SECONDS;
                                        let vcap = sr as usize * ch * DEFAULT_VISUALIZER_SECONDS;
                                        let (_, cons) = make_sample_queue(cap);
                                        audio_state_dp.lock().update_device_config_with_consumer(
                                            BuffConfig::new(cap, DEFAULT_MAX_QUEUE_SECONDS, sr),
                                            BuffConfig::new(vcap, DEFAULT_VISUALIZER_SECONDS, sr),
                                            ch,
                                            cons,
                                        );
                                        dsp_dp.lock().equalizer = Equalizer::new(sr as f32);
                                    }
                                    ctrl_dp.set_state(prev.clone());
                                    if prev == PlaybackState::Playing {
                                        let ms = audio_state_dp.lock().position();
                                        if let Some(tx) = active_cmd_tx_clone.lock().as_ref() {
                                            let _ = tx.send(DecodeCommand::Seek { millis: ms });
                                        }
                                    }
                                }
                                Err(e) => warn!("Failed to rebuild audio output: {e}"),
                            }
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .expect("Can not start device pool!");

        AudioPlayer {
            decode,
            ctrl,
            queue_prod: Mutex::new(Some(queue_prod)),
            audio_state,
            dsp,
            output,
            output_config: out_config,
            active_cmd_tx,
            visualizer_pc: None,
            visualizer_fft_size: 2048,
            device_pool: Some(device_pool),
            device_pool_stop,
        }
    }

    // ── Source & playback control ─────────────────────────────────────────────

    /// Set source for preparing to play.
    #[frb(sync)]
    pub fn set_source(&mut self, source: AudioSource) {
        let is_same = self.decode.source.as_ref().map_or(false, |c| c == &source);
        let playing = self.ctrl.state() == PlaybackState::Playing;

        if is_same && playing {
            warn!("Redundant source while playing");
            return;
        }

        self.decode.signal_stop();
        self.decode.reap_finished();
        *self.active_cmd_tx.lock() = None;
        self.decode.set_source(source);

        // Fresh queue so no stale samples from the old source survive.
        self.rebuild_queue();

        self.audio_state.lock().clear_audio_state();
        self.ctrl.set_state(PlaybackState::Idle);
    }

    /// Play current audio source.
    #[frb(sync)]
    pub fn play(&mut self) {
        self.decode.reap_finished();

        let restart = !self.decode.is_decoding();
        if restart {
            let state = self.ctrl.state();
            if matches!(
                state,
                PlaybackState::Completed | PlaybackState::Stopped | PlaybackState::Idle
            ) {
                self.audio_state.lock().clear_audio_state();
            }
            self.start_decode();
        }

        self.ensure_stream();
        self.ctrl.set_state(PlaybackState::Playing);
    }

    /// Pause current playback.
    #[frb(sync)]
    pub fn pause(&mut self) {
        self.ctrl.set_state(PlaybackState::Paused);
    }

    /// Resume paused playback.
    #[frb(sync)]
    pub fn resume(&mut self) {
        self.ctrl.set_state(PlaybackState::Playing);
    }

    /// Stop playback and reset position.
    #[frb(sync)]
    pub fn stop(&mut self) {
        self.decode.stop();
        *self.active_cmd_tx.lock() = None;
        self.rebuild_queue();
        self.audio_state.lock().clear_audio_state();
        self.ctrl.set_state(PlaybackState::Stopped);
    }

    /// Seek to `position` ms.
    #[frb(sync)]
    pub fn seek(&mut self, position: i32) {
        let mut target = position.max(0);
        if self.decode.duration > 0 {
            target = target.min(self.decode.duration);
        }

        if self.decode.source.is_none() {
            warn!("Seek called with no source");
            return;
        }

        let was_playing = self.ctrl.state() == PlaybackState::Playing;

        self.ensure_stream();
        self.decode.reap_finished();
        if !self.decode.is_decoding() {
            self.start_decode();
        }

        // Drain stale samples, then set position before decode fills the queue.
        {
            let cfg = self.output_config;
            let target_samples = Self::millis_to_samples(target, cfg);
            let mut s = self.audio_state.lock();
            s.clear_buffers();
            s.source_position = target_samples as f64;
        }

        self.decode.seek(target);

        let can_play = self.decode.source.is_some()
            && (self.decode.duration <= 0 || position < self.decode.duration);
        if can_play && !was_playing {
            self.ctrl.set_state(PlaybackState::Playing);
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn millis_to_samples(millis: i32, cfg: AudioOutputConfig) -> u64 {
        (millis.max(0) as u64)
            .saturating_mul(cfg.sample_rate as u64)
            .saturating_mul(cfg.channels as u64)
            / 1000
    }

    fn start_decode(&mut self) {
        if self.decode.is_decoding() {
            info!("Decode thread already running");
            return;
        }

        // Take the current producer, or allocate a new one if already consumed.
        let prod = self.queue_prod.lock().take().unwrap_or_else(|| {
            let cap = self.output_config.sample_rate as usize
                * self.output_config.channels as usize
                * DEFAULT_MAX_QUEUE_SECONDS;
            let (p, cons) = make_sample_queue(cap);
            // Install the matching consumer into AudioState.
            use ringbuf::traits::Consumer;
            let mut s = self.audio_state.lock();
            while s.queue_consumer.try_pop().is_some() {}
            s.queue_consumer = cons;
            p
        });

        let tx = self.decode.start(
            Arc::clone(&self.ctrl),
            prod,
            self.output_config,
            -1, // start from beginning; clear_audio_state() already zeroed position
        );
        *self.active_cmd_tx.lock() = Some(tx);
    }

    /// Allocate a fresh SPSC ring and install the consumer into AudioState.
    /// Stores the producer in self.queue_prod for the next start_decode call.
    fn rebuild_queue(&mut self) {
        let cap = self.output_config.sample_rate as usize
            * self.output_config.channels as usize
            * DEFAULT_MAX_QUEUE_SECONDS;
        let (prod, cons) = make_sample_queue(cap);
        *self.queue_prod.lock() = Some(prod);
        // Drain and replace the consumer inside AudioState.
        use ringbuf::traits::Consumer;
        let mut s = self.audio_state.lock();
        while s.queue_consumer.try_pop().is_some() {}
        s.queue_consumer = cons;
    }

    fn ensure_stream(&mut self) {
        let build_params = {
            let out = self.output.lock();
            out.build_params()
        };
        let Some((tx, state, dsp)) = build_params else {
            return;
        };

        let result = AudioOutput::build_stream(tx, state, dsp);
        if let Ok((stream, new_cfg)) = result {
            {
                let mut out = self.output.lock();
                if out.has_stream() {
                    return; // device_pool won the race; drop our unused stream
                }
                out.install_stream(stream, new_cfg);
            }
            if new_cfg.sample_rate != self.output_config.sample_rate
                || new_cfg.channels != self.output_config.channels
            {
                self.output_config = new_cfg;
                let sr = new_cfg.sample_rate;
                let ch = new_cfg.channels as usize;
                let cap = sr as usize * ch * DEFAULT_MAX_QUEUE_SECONDS;
                let vcap = sr as usize * ch * DEFAULT_VISUALIZER_SECONDS;
                let (prod, cons) = make_sample_queue(cap);
                *self.queue_prod.lock() = Some(prod);
                self.audio_state.lock().update_device_config_with_consumer(
                    BuffConfig::new(cap, DEFAULT_MAX_QUEUE_SECONDS, sr),
                    BuffConfig::new(vcap, DEFAULT_VISUALIZER_SECONDS, sr),
                    ch,
                    cons,
                );
            }
        }
    }

    // ── Volume / rate ─────────────────────────────────────────────────────────

    #[frb(sync)]
    pub fn set_volumn(&mut self, volumn: f32) {
        self.ctrl.set_volume(volumn.clamp(0.0, 1.0));
    }

    #[frb(sync)]
    pub fn set_rate(&mut self, rate: f32) {
        let clamped = rate.clamp(MIN_RATE, MAX_RATE);
        let pos = self.audio_state.lock().position().max(0);
        self.ctrl.set_rate(clamped);
        self.decode.seek(pos);
        {
            let cfg = self.output_config;
            let target_samples = Self::millis_to_samples(pos, cfg);
            let mut s = self.audio_state.lock();
            s.clear_buffers();
            s.source_position = target_samples as f64;
        }
    }

    // ── DSP ───────────────────────────────────────────────────────────────────

    #[frb(sync)]
    pub fn set_10band_eq(&mut self) {
        let sr = self.output_config.sample_rate as f32;
        self.dsp.lock().equalizer = Equalizer::new_graphic_10band(sr);
    }

    #[frb(sync)]
    pub fn add_band(&mut self, config: BandConfig) {
        self.dsp.lock().equalizer.add_band(config);
    }

    #[frb(sync)]
    pub fn set_8band_eq(&mut self) {
        let sr = self.output_config.sample_rate as f32;
        self.dsp.lock().equalizer = Equalizer::new_graphic_8band(sr);
    }

    #[frb(sync)]
    pub fn set_band(&mut self, index: i32, config: BandConfig) {
        self.dsp.lock().equalizer.set_band(index as usize, config);
    }

    #[frb(sync)]
    pub fn set_band_gain(&mut self, index: i32, gain: f32) {
        self.dsp
            .lock()
            .equalizer
            .set_band_gain(index as usize, gain);
    }

    #[frb(sync)]
    pub fn set_delay(&mut self, delay_seconds: f32, feedback: f32, wet: f32) {
        let sr = self.output_config.sample_rate as f32;
        self.dsp.lock().effects.delay = Some(DelayEffect::new(delay_seconds, feedback, wet, sr));
    }

    #[frb(sync)]
    pub fn set_reverb(&mut self, room_size: f32, damping: f32, wet: f32) {
        let sr = self.output_config.sample_rate as f32;
        self.dsp.lock().effects.reverb = Some(ReverbEffect::new(room_size, damping, wet, sr));
    }

    #[frb(sync)]
    pub fn set_chorus(&mut self, rate: f32, depth: f32, wet: f32) {
        let sr = self.output_config.sample_rate as f32;
        self.dsp.lock().effects.chorus = Some(ChorusEffect::new(rate, depth, wet, sr));
    }

    #[frb(sync)]
    pub fn set_threshold(&mut self, threshold: f32) {
        self.dsp.lock().effects.hard_clip = Some(HardClipEffect::new(threshold));
    }

    #[frb(sync)]
    pub fn set_override(&mut self, drive: f32, output: f32) {
        self.dsp.lock().effects.overdrive = Some(OverdriveEffect::new(drive, output));
    }

    #[frb(sync)]
    pub fn clear_effects(&mut self) {
        self.dsp.lock().clear_effects();
    }

    // ── Visualizer ───────────────────────────────────────────────────────────

    #[frb(sync)]
    pub fn set_processor(&mut self, bar_count: i32, fft_size: i32) {
        let fft_size = fft_size.max(512) as usize;
        self.visualizer_fft_size = fft_size;
        self.visualizer_pc = Some(Mutex::new(VisualizerProcessor::new(
            bar_count as usize,
            fft_size,
        )));
    }

    #[frb(sync)]
    pub fn get_bars(&self) -> Vec<f32> {
        let channels = self.output_config.channels as usize;
        let sample_rate = self.output_config.sample_rate;
        let window_samples = self.visualizer_fft_size * channels;

        let (samples, playing) = {
            let s = self.audio_state.lock();
            (
                s.visualizer_window(window_samples),
                self.ctrl.state() == PlaybackState::Playing,
            )
        };

        self.visualizer_pc
            .as_ref()
            .expect("Processor not set")
            .lock()
            .compute(&samples, channels, sample_rate, playing)
    }

    #[frb(sync)]
    pub fn get_band(&self, kinds: Vec<VBandKind>) -> Vec<BandEnergy> {
        self.visualizer_pc
            .as_ref()
            .expect("Processor not set")
            .lock()
            .compute_bands(&kinds)
    }

    #[frb(sync)]
    pub fn get_samples(&self, buff_size: i32) -> Vec<f32> {
        self.audio_state
            .lock()
            .copy_latest_visualizer_samples(buff_size as usize)
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    #[frb(sync)]
    pub fn duration_millis(&self) -> i32 {
        self.decode.duration
    }

    #[frb(sync)]
    pub fn position(&self) -> i32 {
        self.audio_state.lock().position()
    }

    #[frb(sync)]
    pub fn get_state(&self) -> i32 {
        self.ctrl.state().id()
    }

    #[frb(sync)]
    pub fn is_playing(&self) -> bool {
        self.ctrl.state() == PlaybackState::Playing
    }

    #[frb(sync)]
    pub fn is_completed(&self) -> bool {
        self.ctrl.state() == PlaybackState::Completed
    }

    #[frb(sync)]
    pub fn sample_rate(&self) -> f32 {
        self.output_config.sample_rate as f32
    }

    #[frb(sync)]
    pub fn get_output_config(&self) -> AudioOutputConfig {
        self.output.lock().current_config()
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.decode.stop();
        self.device_pool_stop.store(true, Ordering::SeqCst);
        if let Some(pool) = self.device_pool.take() {
            let _ = pool.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let player = AudioPlayer::new();
        assert_eq!(player.get_state(), 0); // Idle
    }

    #[test]
    fn test_set_source_and_play_flow() {
        let mut player = AudioPlayer::new();
        assert_eq!(player.get_state(), 0);

        player.set_source(AudioSource::Path("test.mp3".to_string()));
        assert_eq!(player.get_state(), 0); // Idle

        player.play();
        assert_eq!(player.get_state(), 2); // Playing

        player.set_source(AudioSource::Path("test2.mp3".to_string()));
        assert_eq!(player.get_state(), 0); // Idle

        player.play();
        assert_eq!(player.get_state(), 2); // Playing

        player.stop();
        assert_eq!(player.get_state(), 4); // Stopped

        player.set_source(AudioSource::Path("test2.mp3".to_string()));
        assert_eq!(player.get_state(), 0); // Idle
        player.play();
        assert_eq!(player.get_state(), 2); // Playing
    }

    #[test]
    fn test_effect_setters_no_deadlock() {
        // All of these previously locked state-then-output (wrong order → deadlock).
        let mut player = AudioPlayer::new();
        player.set_10band_eq();
        player.set_delay(0.3, 0.5, 0.4);
        player.set_reverb(0.5, 0.5, 0.3);
        player.set_chorus(1.0, 0.5, 0.3);
    }
}
