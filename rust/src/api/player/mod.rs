use std::{
    sync::{Arc, atomic::AtomicBool},
    thread,
};

use crossbeam_channel::{Receiver, Sender, unbounded};
use flutter_rust_bridge::frb;
use parking_lot::Mutex;

use crate::{
    api::{
        enums::{AudioCommand, MAX_RATE, MIN_RATE},
        filters::{
            effects::{ChorusEffect, DelayEffect, HardClipEffect, OverdriveEffect, ReverbEffect},
            equalizer::Equalizer,
        },
        renderer::{
            output::{AudioOutput, AudioOutputConfig},
            state::PlaybackState,
        },
        source::{
            AudioSource,
            decode::{DecodeCommand, DecodePool},
        },
        visualizer::{BandEnergy, VBandKind, VisualizerProcessor},
    },
    info, warn,
};

use crate::api::{
    enums::{DEFAULT_MAX_QUEUE_SECONDS, DEFAULT_VISUALIZER_SECONDS},
    renderer::state::{AudioState, BuffConfig},
};

#[frb(opaque)]
pub struct AudioPlayer {
    decode: DecodePool,
    output: Arc<Mutex<AudioOutput>>,
    state: Arc<Mutex<AudioState>>,
    cmd_rc: Receiver<DecodeCommand>,
    visualizer_pc: Option<Mutex<VisualizerProcessor>>,
    visualizer_fft_size: usize,
    output_config: AudioOutputConfig,
    device_pool: Option<thread::JoinHandle<()>>,
    device_pool_stop: Arc<AtomicBool>,
    cmd_tx: Sender<DecodeCommand>,
}

impl AudioPlayer {
    /// Create new player instance.
    #[frb(sync)]
    pub fn new() -> AudioPlayer {
        let decode = DecodePool::new();

        let (tx, rc) = unbounded();

        let out_config = AudioOutput::get_output_config();

        let sample_rate = out_config.sample_rate;

        let channels = out_config.channels;

        let max_samples = (sample_rate as usize)
            .saturating_mul(channels as usize)
            .saturating_mul(DEFAULT_MAX_QUEUE_SECONDS);

        let v_max_samples = (sample_rate as usize)
            .saturating_mul(channels as usize)
            .saturating_mul(DEFAULT_VISUALIZER_SECONDS);

        let config = BuffConfig::new(max_samples, DEFAULT_MAX_QUEUE_SECONDS, sample_rate);

        let v_config = BuffConfig::new(v_max_samples, DEFAULT_VISUALIZER_SECONDS, sample_rate);

        let state = Arc::new(Mutex::new(AudioState::new(
            config,
            v_config,
            channels as usize,
        )));

        let output = Arc::new(Mutex::new(AudioOutput::new(
            tx,
            Arc::clone(&state),
            out_config,
        )));

        let state_for_callback = Arc::clone(&state);

        let (cmd_tx, cmd_rc) = unbounded();

        let dc_tx_clone = cmd_tx.clone();

        let output_config = AudioOutput::get_output_config();

        let rc_clone = rc.clone();

        let output_clone = Arc::clone(&output);

        let device_pool_stop = Arc::new(AtomicBool::new(false));

        let stop_flag = Arc::clone(&device_pool_stop);

        let device_pool = thread::Builder::new()
            .name("Device Pool".to_string())
            .spawn(move || {
                loop {
                    if stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }

                    while let Ok(cmd) = rc_clone.try_recv() {
                        match cmd {
                            AudioCommand::RebuildOuput => {
                                state_for_callback.lock().pl_state = PlaybackState::Paused;

                                let mut mvd_output = output_clone.lock();
                                mvd_output.dispose_old_stream();

                                match mvd_output.build() {
                                    Ok(new_cfg) => {
                                        if new_cfg.sample_rate != output_config.sample_rate
                                            || new_cfg.channels != output_config.channels
                                        {
                                            let mut s = state_for_callback.lock();
                                            s.update_device_config(
                                                BuffConfig::new(
                                                    (new_cfg.sample_rate as usize)
                                                        .saturating_mul(new_cfg.channels as usize)
                                                        .saturating_mul(DEFAULT_MAX_QUEUE_SECONDS),
                                                    DEFAULT_MAX_QUEUE_SECONDS,
                                                    new_cfg.sample_rate,
                                                ),
                                                BuffConfig::new(
                                                    (new_cfg.sample_rate as usize)
                                                        .saturating_mul(new_cfg.channels as usize)
                                                        .saturating_mul(DEFAULT_VISUALIZER_SECONDS),
                                                    DEFAULT_VISUALIZER_SECONDS,
                                                    new_cfg.sample_rate,
                                                ),
                                                new_cfg.channels as usize,
                                            );
                                        }
                                        state_for_callback.lock().pl_state = PlaybackState::Playing;

                                        let millis = state_for_callback.lock().position();

                                        let _ = dc_tx_clone.send(DecodeCommand::Seek { millis });
                                    }
                                    Err(e) => {
                                        warn!("Failed to rebuild audio output: {e}");
                                    }
                                }
                            }
                        }
                    }
                }
            })
            .expect("Can not start device pool!");

        AudioPlayer {
            decode,
            output,
            cmd_rc,
            state,
            visualizer_pc: None,
            visualizer_fft_size: 2048,
            output_config: out_config,
            device_pool: Some(device_pool),
            device_pool_stop,
            cmd_tx,
        }
    }

    /// Set source for preparing to play.
    #[frb(sync)]
    pub fn set_source(&mut self, source: AudioSource) {
        self.decode.stop();
        self.decode.set_source(source);
        let mut s = self.state.lock();
        s.clear_audio_state();
        s.stream_ended = false;
        s.pl_state = PlaybackState::Idle;
    }

    /// Play current audio source. If current source is empty then an error will be throw.
    #[frb(sync)]
    pub fn play(&mut self) {
        self.ensure_stream();
        self.decode.reap_finished();

        let restart_decode = !self.decode.is_decoding();
        if restart_decode {
            let mut s = self.state.lock();
            s.stream_ended = false;
            if matches!(
                s.pl_state,
                PlaybackState::Completed | PlaybackState::Stopped
            ) {
                s.clear_audio_state();
            }
        }

        if restart_decode {
            self.start_decode();
        }
        // set state to playing
        self.resume();
        self.decode.seek(1);
    }

    fn start_decode(&mut self) {
        if self.decode.is_decoding() {
            info!("Decode thread already running");
            return;
        }

        info!("Creating new decoder!");

        let state = Arc::clone(&self.state);
        let output_config = self.output_config;

        self.decode.start(
            state,
            output_config,
            0,
            self.cmd_tx.clone(),
            self.cmd_rc.clone(),
        );
    }

    /// Pause curent playing song.
    #[frb(sync)]
    pub fn pause(&mut self) {
        self.state.lock().pl_state = PlaybackState::Paused;
    }

    #[frb(sync)]
    pub fn resume(&mut self) {
        self.state.lock().pl_state = PlaybackState::Playing;
    }

    #[frb(sync)]
    pub fn is_completed(&self) -> bool {
        self.state.lock().pl_state == PlaybackState::Completed
    }

    /// Whether player is playing or not.
    #[frb(sync)]
    pub fn is_playing(&self) -> bool {
        self.state.lock().pl_state == PlaybackState::Playing
    }

    #[frb(sync)]
    pub fn sample_rate(&self) -> f32 {
        self.output.lock().current_config().sample_rate as f32
    }

    /// Play audio at given position in millisecond.
    #[frb(sync)]
    pub fn seek(&mut self, position: i32) {
        let mut target = position.max(0);
        if self.decode.duration > 0 {
            target = target.min(self.decode.duration);
        }

        let was_playing = self.is_playing();

        if !matches!(self.decode.source, Some(_)) {
            warn!("Seek called with no source");
            return;
        }

        self.ensure_stream();
        self.decode.reap_finished();
        if !self.decode.is_decoding() {
            self.start_decode();
        }

        self.decode.seek(position);

        let output_config = self.output_config;
        let target_samples = ((target as u64)
            .saturating_mul(output_config.sample_rate as u64)
            .saturating_mul(output_config.channels as u64)
            / 1000) as u64;

        {
            let mut s = self.state.lock();
            s.queue.clear();
            s.visualizer.clear();
            s.emitted_samples = 0;
            s.source_position = target_samples as f64;
            s.stream_ended = false;
        }

        let can_play = self.decode.source.is_some()
            && (self.decode.duration <= 0 || position < self.decode.duration);

        if can_play && !was_playing {
            self.resume();
        }
    }

    fn ensure_stream(&mut self) {
        let mut out = self.output.lock();
        if let Ok(new_cfg) = out.build() {
            if new_cfg.sample_rate != self.output_config.sample_rate
                || new_cfg.channels != self.output_config.channels
            {
                self.output_config = new_cfg;
                let mut s = self.state.lock();
                s.update_device_config(
                    BuffConfig::new(
                        (new_cfg.sample_rate as usize)
                            .saturating_mul(new_cfg.channels as usize)
                            .saturating_mul(DEFAULT_MAX_QUEUE_SECONDS),
                        DEFAULT_MAX_QUEUE_SECONDS,
                        new_cfg.sample_rate,
                    ),
                    BuffConfig::new(
                        (new_cfg.sample_rate as usize)
                            .saturating_mul(new_cfg.channels as usize)
                            .saturating_mul(DEFAULT_VISUALIZER_SECONDS),
                        DEFAULT_VISUALIZER_SECONDS,
                        new_cfg.sample_rate,
                    ),
                    new_cfg.channels as usize,
                );
            }
        }
    }

    /// Stop current playing audio.
    #[frb(sync)]
    pub fn stop(&mut self) {
        self.decode.stop();
        let mut s = self.state.lock();
        s.clear_audio_state();
        s.pl_state = PlaybackState::Stopped;
    }

    /// Set volume of ouput audio.
    #[frb(sync)]
    pub fn set_volumn(&mut self, volumn: f32) {
        self.state.lock().volumn = volumn.clamp(0.0, 1.0);
    }

    /// Set playback speed of output audio.
    #[frb(sync)]
    pub fn set_rate(&mut self, rate: f32) {
        let (resume_position, clamped_rate) = {
            let s = self.state.lock();
            (s.position().max(0), rate.clamp(MIN_RATE, MAX_RATE))
        };

        self.decode.seek(resume_position);

        let mut s = self.state.lock();
        s.pl_rate = clamped_rate;
        s.queue.clear();
        s.visualizer.clear();
        s.stream_ended = false;
    }

    #[frb(sync)]
    pub fn set_10band_eq(&mut self) {
        self.state.lock().equalizer = Equalizer::new_graphic_10band(self.sample_rate())
    }

    #[frb(sync)]
    pub fn set_band_gain(&mut self, index: i32, gain: f32) {
        self.state
            .lock()
            .equalizer
            .set_band_gain(index as usize, gain);
    }

    #[frb(sync)]
    pub fn set_delay(&mut self, delay_seconds: f32, feedback: f32, wet: f32) {
        self.state.lock().effects.delay = Some(DelayEffect::new(
            delay_seconds,
            feedback,
            wet,
            self.sample_rate(),
        ));
    }

    #[frb(sync)]
    pub fn set_reverb(&mut self, room_size: f32, damping: f32, wet: f32) {
        self.state.lock().effects.reverb = Some(ReverbEffect::new(
            room_size,
            damping,
            wet,
            self.sample_rate(),
        ))
    }

    #[frb(sync)]
    pub fn set_chorus(&mut self, rate: f32, depth: f32, wet: f32) {
        self.state.lock().effects.chorus =
            Some(ChorusEffect::new(rate, depth, wet, self.sample_rate()))
    }

    #[frb(sync)]
    pub fn set_threshold(&mut self, threshold: f32) {
        self.state.lock().effects.hard_clip = Some(HardClipEffect::new(threshold));
    }

    #[frb(sync)]
    pub fn set_override(&mut self, drive: f32, output: f32) {
        self.state.lock().effects.overdrive = Some(OverdriveEffect::new(drive, output));
    }

    #[frb(sync)]
    pub fn set_processor(&mut self, bar_count: i32, fft_size: i32) {
        let fft_size = fft_size.max(512) as usize;
        self.visualizer_fft_size = fft_size;
        self.visualizer_pc = Some(Mutex::new(VisualizerProcessor::new(
            bar_count as usize,
            fft_size,
        )));
    }

    /// Duration in millisecond of current source.
    #[frb(sync)]
    pub fn duration_millis(&self) -> i32 {
        self.decode.duration
    }

    /// Clear the effect chain of every output channel.
    #[frb(sync)]
    pub fn clear_effects(&mut self) {
        self.state.lock().clear_effects();
    }

    #[frb(sync)]
    pub fn position(&self) -> i32 {
        self.state.lock().position()
    }

    #[frb(sync)]
    pub fn get_state(&self) -> i32 {
        self.state.lock().pl_state.id()
    }

    #[frb(sync)]
    pub fn get_bars(&self) -> Vec<f32> {
        let channels = self.output_config.channels as usize;
        let sample_rate = self.output_config.sample_rate;
        let window_samples = self.visualizer_fft_size * channels;

        let (samples, playing) = {
            let s = self.state.lock();
            (
                s.visualizer_window(window_samples),
                s.pl_state == PlaybackState::Playing,
            )
        };

        let processor = self.visualizer_pc.as_ref().expect("Processor didn't set");
        processor
            .lock()
            .compute(&samples, channels, sample_rate, playing)
    }

    #[frb(sync)]
    pub fn get_band(&self, kinds: Vec<VBandKind>) -> Vec<BandEnergy> {
        self.visualizer_pc
            .as_ref()
            .expect("Processor didn't set")
            .lock()
            .compute_bands(&kinds)
    }

    #[frb(sync)]
    pub fn get_samples(&self, buff_size: i32) -> Vec<f32> {
        self.state
            .lock()
            .copy_latest_visualizer_samples(buff_size as usize)
    }

    #[frb(sync)]
    pub fn get_output_config(&self) -> AudioOutputConfig {
        self.output.lock().current_config()
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.device_pool_stop
            .store(true, std::sync::atomic::Ordering::SeqCst);
        if let Some(pool) = self.device_pool.take() {
           let _ = pool.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use super::*;

    #[test]
    fn it_works() {
        let mut player = AudioPlayer::new();

        player.set_source(AudioSource::Path(
            "/home/thuong/Downloads/oblivion/alwaysbe.mp3".to_string(),
        ));

        player.play();

        sleep(Duration::from_mins(2));
    }
}
