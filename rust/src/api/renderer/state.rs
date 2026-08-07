use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering},
    },
    time::Duration,
};

use crate::api::filters::{effects::EffectsChain, equalizer::Equalizer};
use flutter_rust_bridge::frb;
use ringbuf::{HeapRb, traits::Split};

// ─────────────────────────────────────────────────────────────────────────────
// PlaybackState
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Idle,
    Buffering,
    Playing,
    Paused,
    Stopped,
    Completed,
}

impl PlaybackState {
    pub fn id(&self) -> i32 {
        match self {
            PlaybackState::Idle => 0,
            PlaybackState::Buffering => 1,
            PlaybackState::Playing => 2,
            PlaybackState::Paused => 3,
            PlaybackState::Stopped => 4,
            PlaybackState::Completed => 5,
        }
    }

    pub fn to_u8(&self) -> u8 {
        self.id() as u8
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => PlaybackState::Idle,
            1 => PlaybackState::Buffering,
            2 => PlaybackState::Playing,
            3 => PlaybackState::Paused,
            4 => PlaybackState::Stopped,
            5 => PlaybackState::Completed,
            _ => PlaybackState::Idle,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ResampleState
// ─────────────────────────────────────────────────────────────────────────────

#[frb(opaque)]
pub struct ResampleState {
    pub(crate) pos: f64,
    pub(crate) carry: Vec<f32>,
}

// SAFETY: ResampleState only contains f64 + Vec<f32>, both Sync.
unsafe impl Sync for ResampleState {}

impl ResampleState {
    pub fn new() -> Self {
        Self {
            pos: 0.0,
            carry: Vec::new(),
        }
    }
    pub fn reset(&mut self) {
        self.pos = 0.0;
        self.carry.clear();
    }
}

impl Default for ResampleState {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BuffConfig
// ─────────────────────────────────────────────────────────────────────────────
#[frb(opaque)]
pub struct BuffConfig {
    pub sample_rate: u32,
    pub max_samples: usize,
    pub m_queue_sec: usize,
}

impl BuffConfig {
    pub fn new(max_samples: usize, m_queue_sec: usize, sample_rate: u32) -> BuffConfig {
        BuffConfig {
            sample_rate,
            max_samples,
            m_queue_sec,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SharedControlState — lock-free atomics
// ─────────────────────────────────────────────────────────────────────────────

#[frb(opaque)]
pub(crate) struct SharedControlState {
    pub(crate) pl_state: AtomicU8,
    pub(crate) pl_rate: AtomicU32,
    pub(crate) volume: AtomicU32,
    pub(crate) stream_ended: AtomicBool,
}

impl SharedControlState {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            pl_state: AtomicU8::new(PlaybackState::Idle.to_u8()),
            pl_rate: AtomicU32::new(1.0f32.to_bits()),
            volume: AtomicU32::new(1.0f32.to_bits()),
            stream_ended: AtomicBool::new(false),
        })
    }

    #[inline]
    pub fn state(&self) -> PlaybackState {
        PlaybackState::from_u8(self.pl_state.load(Ordering::Acquire))
    }
    #[inline]
    pub fn set_state(&self, s: PlaybackState) {
        self.pl_state.store(s.to_u8(), Ordering::Release);
    }
    #[inline]
    pub fn rate(&self) -> f32 {
        f32::from_bits(self.pl_rate.load(Ordering::Relaxed))
    }
    #[inline]
    pub fn set_rate(&self, r: f32) {
        self.pl_rate.store(r.to_bits(), Ordering::Relaxed);
    }
    #[inline]
    pub fn get_volume(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }
    #[inline]
    pub fn set_volume(&self, v: f32) {
        self.volume.store(v.to_bits(), Ordering::Relaxed);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Lock-free SPSC queue
// ─────────────────────────────────────────────────────────────────────────────

pub(crate) fn make_sample_queue(capacity: usize) -> (ringbuf::HeapProd<f32>, ringbuf::HeapCons<f32>) {
    HeapRb::<f32>::new(capacity.max(1)).split()
}

// ─────────────────────────────────────────────────────────────────────────────
// DspState — EQ + effects behind its own Mutex in AudioPlayer
// ─────────────────────────────────────────────────────────────────────────────

pub struct DspState {
    pub effects: EffectsChain,
    pub equalizer: Equalizer,
}

impl DspState {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            effects: EffectsChain::empty(),
            equalizer: Equalizer::new(sample_rate),
        }
    }
    pub fn clear_effects(&mut self) {
        self.effects = EffectsChain::empty();
    }
    pub fn clear_eq(&mut self) {
        self.equalizer.clear_bands();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AudioState
//
// Keeps backward-compatible public methods so frb_generated.rs keeps compiling
// without regeneration.  The audio hot-path uses atomics (ctrl) + a lock-free
// SPSC queue (queue_consumer) so the Mutex<AudioState> is almost never
// contended between real-time threads.
// ─────────────────────────────────────────────────────────────────────────────

#[frb(opaque)]
pub struct AudioState {
    pub(crate) ctrl: Arc<SharedControlState>,

    pub(crate) source_position: f64,
    pub(crate) emitted_samples: u64,
    pub(crate) underrun_count: u32,

    /// SPSC consumer half — audio callback pops samples lock-free.
    pub(crate) queue_consumer: ringbuf::HeapCons<f32>,

    pub(crate) visualizer: VecDeque<f32>,

    // ── Backward-compat DSP fields kept for frb_generated.rs ─────────────────
    // frb_generated accesses these directly as public fields.  AudioPlayer
    // uses DspState (its own Mutex) for the actual audio path, but when frb
    // calls `AudioState::set_equalizer` we store the value here too so that
    // any frb-level round-trips remain consistent.
    pub effects: EffectsChain,
    pub equalizer: Equalizer,

    config: BuffConfig,
    v_config: BuffConfig,
    pub(crate) channels: usize,
}

// SAFETY: AudioState is always accessed through Mutex<AudioState>.
// ringbuf::HeapCons<f32>; uses Cell<usize> (non-Sync) but that is serialised by the mutex.
unsafe impl Sync for AudioState {}

impl AudioState {
    /// Constructor called by frb_generated (3-arg legacy signature).
    pub fn new(config: BuffConfig, v_config: BuffConfig, channels: usize) -> AudioState {
        let ctrl = SharedControlState::new();
        let (_, cons) = make_sample_queue(config.max_samples.max(1));
        AudioState::new_with_ctrl(ctrl, cons, config, v_config, channels)
    }

    /// Internal constructor used by AudioPlayer.
    pub(crate) fn new_with_ctrl(
        ctrl: Arc<SharedControlState>,
        queue_consumer: ringbuf::HeapCons<f32>,
        config: BuffConfig,
        v_config: BuffConfig,
        channels: usize,
    ) -> Self {
        let sr = config.sample_rate as f32;
        AudioState {
            ctrl,
            source_position: 0.0,
            emitted_samples: 0,
            underrun_count: 0,
            queue_consumer,
            visualizer: VecDeque::with_capacity(v_config.max_samples),
            effects: EffectsChain::empty(),
            equalizer: Equalizer::new(sr),
            channels,
            config,
            v_config,
        }
    }

    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        let mut s = [0.0f32; 1];
        self.render_output_f32(&mut s, None);
        s[0]
    }

    /// Render audio samples. Called from the real-time audio callback (no blocking).
    pub(crate) fn render_output_f32(&mut self, data: &mut [f32], dsp: Option<&mut DspState>) {
        use ringbuf::traits::{Consumer, Observer};

        if self.ctrl.state() != PlaybackState::Playing {
            data.fill(0.0);
            return;
        }

        let total = data.len();
        let avail = self.queue_consumer.occupied_len().min(total);

        if avail == 0 {
            data.fill(0.0);
            if self.ctrl.stream_ended.load(Ordering::Acquire) {
                self.ctrl.set_state(PlaybackState::Completed);
            } else {
                self.underrun_count = self.underrun_count.saturating_add(total as u32);
            }
            return;
        }

        // Batch-pop from the lock-free consumer.
        self.queue_consumer.pop_slice(&mut data[..avail]);
        data[avail..].fill(0.0);

        self.emitted_samples = self.emitted_samples.saturating_add(avail as u64);
        self.source_position += avail as f64 * self.ctrl.rate() as f64;

        if let Some(dsp) = dsp {
            let vol = self.ctrl.get_volume();
            for s in data[..avail].iter_mut() {
                let x = *s * vol;
                let x = dsp.equalizer.process(x);
                let x = dsp.effects.process(x);
                *s = x.clamp(-1.0, 1.0);
            }
        }

        self.push_visualizer_samples(&data[..avail]);
    }

    pub fn position(&self) -> i32 {
        if self.config.sample_rate == 0 {
            return 0;
        }
        let secs = self.source_position / (self.config.sample_rate as f64 * self.channels as f64);
        Duration::from_secs_f64(secs.max(0.0)).as_millis() as i32
    }

    pub fn clear_buffers(&mut self) {
        use ringbuf::traits::Consumer;
        while self.queue_consumer.try_pop().is_some() {}
        self.visualizer.clear();
        self.emitted_samples = 0;
        self.underrun_count = 0;
        self.ctrl.stream_ended.store(false, Ordering::Release);
    }

    pub fn clear_audio_state(&mut self) {
        self.clear_buffers();
        self.source_position = 0.0;
    }

    // ── Backward-compat methods called by frb_generated ───────────────────────

    pub fn clear_effects(&mut self) {
        self.effects = EffectsChain::empty();
    }

    pub fn clear_eq(&mut self) {
        self.equalizer.clear_bands();
    }

    pub fn set_equalizer(&mut self, equalizer: Equalizer) {
        self.equalizer = equalizer;
    }

    pub fn set_effect(&mut self, effects: EffectsChain) {
        self.effects = effects;
    }

    pub fn push_visualizer_sample(&mut self, sample: f32) {
        self.push_visualizer_samples(&[sample]);
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    pub(crate) fn push_visualizer_samples(&mut self, samples: &[f32]) {
        if self.v_config.max_samples == 0 || samples.is_empty() {
            return;
        }
        if samples.len() >= self.v_config.max_samples {
            self.visualizer.clear();
            let start = samples.len() - self.v_config.max_samples;
            self.visualizer.extend(samples[start..].iter().copied());
        } else {
            let overflow =
                (self.visualizer.len() + samples.len()).saturating_sub(self.v_config.max_samples);
            if overflow > 0 {
                self.visualizer.drain(..overflow);
            }
            self.visualizer.extend(samples.iter().copied());
        }
    }

    /// Legacy 3-arg version called by frb_generated.rs.
    pub fn update_device_config(
        &mut self,
        config: BuffConfig,
        v_config: BuffConfig,
        channels: usize,
    ) {
        use ringbuf::traits::Consumer;
        // Allocate a new ring of the correct size.
        let (_, cons) = make_sample_queue(config.max_samples.max(1));
        while self.queue_consumer.try_pop().is_some() {}
        self.queue_consumer = cons;
        self.config = config;
        self.v_config = v_config;
        self.channels = channels;
        self.visualizer = VecDeque::with_capacity(self.v_config.max_samples);
    }

    /// Extended version used internally when AudioPlayer rebuilds the queue.
    pub(crate) fn update_device_config_with_consumer(
        &mut self,
        config: BuffConfig,
        v_config: BuffConfig,
        channels: usize,
        new_consumer: ringbuf::HeapCons<f32>,
    ) {
        use ringbuf::traits::Consumer;
        while self.queue_consumer.try_pop().is_some() {}
        self.queue_consumer = new_consumer;
        self.config = config;
        self.v_config = v_config;
        self.channels = channels;
        self.visualizer = VecDeque::with_capacity(self.v_config.max_samples);
    }

    pub fn visualizer_window(&self, max_samples: usize) -> Vec<f32> {
        let start = self.visualizer.len().saturating_sub(max_samples);
        self.visualizer.iter().skip(start).copied().collect()
    }

    pub fn copy_latest_visualizer_samples(&self, size: usize) -> Vec<f32> {
        if size == 0 {
            return vec![];
        }
        if self.visualizer.is_empty() {
            return vec![0.0; size];
        }
        let mut out = vec![0.0f32; size];
        let count = size.min(self.visualizer.len());
        let skip = self.visualizer.len().saturating_sub(count);
        let off = size - count;
        for (i, s) in self.visualizer.iter().skip(skip).take(count).enumerate() {
            out[off + i] = *s;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_buffers_preserves_position() {
        let ctrl = SharedControlState::new();
        let (_, cons) = make_sample_queue(1024);
        let mut state = AudioState::new_with_ctrl(
            ctrl,
            cons,
            BuffConfig::new(1024, 1, 44100),
            BuffConfig::new(512, 1, 44100),
            2,
        );
        state.source_position = 44100.0 * 2.0 * 5.0;
        state.clear_buffers();
        assert_eq!(state.position(), 5000);
        state.clear_audio_state();
        assert_eq!(state.position(), 0);
    }
}
