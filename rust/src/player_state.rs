use std::collections::VecDeque;
use std::time::Duration;

use crate::effects::Effects;
use crate::enums::{
    DEFAULT_MAX_QUEUE_SECONDS, DEFAULT_VISUALIZER_SECONDS, MAX_MAX_QUEUE_SECONDS,
    MIN_MAX_QUEUE_SECONDS,
};
use crate::error::AudioError;

// ── PlaybackStatus ────────────────────────────────────────────────────────────

/// Full playback state with more detail than a simple boolean.
///
/// Matches the architecture spec's `PlaybackStatus` enum.
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackStatus {
    /// No source is loaded; the engine is doing nothing.
    Idle,
    /// Audio is actively playing.
    Playing,
    /// Playback is paused — position is preserved.
    Paused,
    /// Waiting for the network / decode buffer to fill.
    Buffering,
    /// The source reached its end naturally.
    Finished,
    /// An error caused playback to stop.
    Error(AudioError),
}

/// Simplified `PlayerState` kept for the public FFI layer (maps to i32).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Idle,
    Playing,
    Paused,
    Stopped,
}

impl From<&PlaybackStatus> for PlayerState {
    fn from(status: &PlaybackStatus) -> Self {
        match status {
            PlaybackStatus::Idle | PlaybackStatus::Buffering => PlayerState::Idle,
            PlaybackStatus::Playing => PlayerState::Playing,
            PlaybackStatus::Paused => PlayerState::Paused,
            PlaybackStatus::Finished | PlaybackStatus::Error(_) => PlayerState::Stopped,
        }
    }
}

// ── ResampleState ─────────────────────────────────────────────────────────────

/// Carries fractional position and boundary samples across decode packets.
///
/// Linear interpolation is used to convert between the source sample rate and
/// the device's native rate (and playback speed).  This state prevents audible
/// glitches at packet boundaries.
pub struct ResampleState {
    /// Current fractional position within the current packet (source frames).
    pub pos: f64,
    /// The last *source* frame of the previous packet, used as the left
    /// neighbour for the first interpolated output sample of the next packet.
    pub carry: Vec<f32>,
}

impl ResampleState {
    pub fn new() -> Self {
        Self { pos: 0.0, carry: Vec::new() }
    }

    pub fn reset(&mut self) {
        self.pos = 0.0;
        self.carry.clear();
    }
}

impl Default for ResampleState {
    fn default() -> Self { Self::new() }
}

// ── SharedPlayback ────────────────────────────────────────────────────────────

/// State that is **shared** between the audio-callback thread and any thread
/// that drives the engine (decode thread, FFI calls, event loop).
///
/// All fields are accessed under a `Mutex` owned by `AudioEngine`.  The lock
/// is held only for the shortest possible time to avoid priority inversion.
pub struct SharedPlayback {
    // ── Sample queues ─────────────────────────────────────────────────────
    /// Ready-to-play interleaved `f32` samples fed by the decode thread and
    /// consumed by the cpal callback.
    pub queue: VecDeque<f32>,

    /// Recent samples kept for visualiser use.  Written by the cpal callback
    /// after applying volume; read by the visualiser on the UI thread.
    pub visualizer_ring: VecDeque<f32>,

    // ── Queue sizing ──────────────────────────────────────────────────────
    pub max_samples:          usize,
    pub max_queue_seconds:    usize,
    pub visualizer_max_samples: usize,

    // ── Playback parameters ───────────────────────────────────────────────
    /// Current playback rate.  1.0 = normal speed.
    pub playback_rate: f32,

    /// Master volume: 0.0 = silence, 1.0 = unity.
    pub volume: f32,

    // ── Playback state flags ──────────────────────────────────────────────
    /// The engine is actively consuming from `queue`.
    pub playing: bool,
    /// The decode thread has written all packets; no more data is coming.
    pub stream_finished: bool,

    // ── Position tracking ─────────────────────────────────────────────────
    /// Total interleaved samples consumed by the cpal callback since
    /// the last seek.  Used to compute `position_millis`.
    pub emitted_samples: u64,

    /// Absolute source position in interleaved output samples.
    ///
    /// Advanced by `playback_rate` per emitted sample so that live rate
    /// changes are reflected in the position without re-scaling past time.
    pub source_position_samples: f64,

    /// Sample rate of the output device (Hz).
    pub sample_rate: u32,

    // ── Per-channel DSP ───────────────────────────────────────────────────
    /// One `Effects` chain per output channel.  Indexed by
    /// `emitted_samples % effects.len()`.
    pub effects: Vec<Effects>,

    // ── Detailed status ───────────────────────────────────────────────────
    /// Fine-grained playback status used for event emission.
    pub status: PlaybackStatus,

    // ── Underrun counter ──────────────────────────────────────────────────
    /// Cumulative buffer-underrun count since last reset.
    pub underrun_count: u32,
}

impl SharedPlayback {
    /// Construct with device-native sample rate and channel count.
    pub fn new(channels: usize, sample_rate: u32) -> Self {
        let max_samples = (sample_rate as usize)
            .saturating_mul(channels)
            .saturating_mul(DEFAULT_MAX_QUEUE_SECONDS);
        let visualizer_max_samples = (sample_rate as usize)
            .saturating_mul(channels)
            .saturating_mul(DEFAULT_VISUALIZER_SECONDS);

        Self {
            queue:                   VecDeque::with_capacity(max_samples),
            visualizer_ring:         VecDeque::with_capacity(visualizer_max_samples),
            visualizer_max_samples,
            max_samples,
            max_queue_seconds:       DEFAULT_MAX_QUEUE_SECONDS,
            playback_rate:           1.0,
            volume:                  1.0,
            playing:                 false,
            stream_finished:         true,
            emitted_samples:         0,
            source_position_samples: 0.0,
            sample_rate,
            effects:                 (0..channels.max(1)).map(|_| Effects::new()).collect(),
            status:                  PlaybackStatus::Idle,
            underrun_count:          0,
        }
    }

    // ── Queue helpers ─────────────────────────────────────────────────────

    /// Push up to `max_samples - queue.len()` samples.  Returns the count
    /// actually pushed; caller retries the rest after sleeping.
    pub fn push_samples_bounded(&mut self, samples: &[f32]) -> usize {
        let available = self.max_samples.saturating_sub(self.queue.len());
        let push_count = samples.len().min(available);
        self.queue.extend(samples.iter().take(push_count).copied());
        push_count
    }

    /// Resize the queue cap.  Excess samples are dropped from the front.
    pub fn set_max_queue_seconds(&mut self, channels: usize, seconds: usize) {
        let bounded = seconds.clamp(MIN_MAX_QUEUE_SECONDS, MAX_MAX_QUEUE_SECONDS);
        self.max_queue_seconds = bounded;
        self.max_samples = (self.sample_rate as usize)
            .saturating_mul(channels)
            .saturating_mul(bounded);
        while self.queue.len() > self.max_samples {
            self.queue.pop_front();
        }
        if self.queue.capacity() < self.max_samples {
            self.queue.reserve(self.max_samples.saturating_sub(self.queue.capacity()));
        }
    }

    // ── Visualiser helpers ────────────────────────────────────────────────

    pub fn push_visualizer_sample(&mut self, sample: f32) {
        if self.visualizer_max_samples == 0 {
            return;
        }
        if self.visualizer_ring.len() >= self.visualizer_max_samples {
            self.visualizer_ring.pop_front();
        }
        self.visualizer_ring.push_back(sample);
    }

    /// Copy the most recent `out.len()` samples from the visualiser ring into
    /// `out`.  Returns the number of samples written.
    pub fn copy_latest_visualizer_samples(&self, out: &mut [f32]) -> usize {
        if out.is_empty() || self.visualizer_ring.is_empty() {
            return 0;
        }
        let count = out.len().min(self.visualizer_ring.len());
        let skip  = self.visualizer_ring.len().saturating_sub(count);
        for (i, s) in self.visualizer_ring.iter().skip(skip).take(count).enumerate() {
            out[i] = *s;
        }
        count
    }

    // ── State reset ───────────────────────────────────────────────────────

    /// Clear all transient audio state without touching volume / rate / device.
    pub fn clear_audio_state(&mut self) {
        self.queue.clear();
        self.visualizer_ring.clear();
        self.emitted_samples         = 0;
        self.source_position_samples = 0.0;
        self.stream_finished         = true;
        self.underrun_count          = 0;
        for e in &mut self.effects {
            *e = Effects::new();
        }
    }

    // ── Hot-path sample output ────────────────────────────────────────────

    /// Called by the cpal callback for every output sample.
    ///
    /// Returns 0.0 (silence) if paused, buffering, or the queue is empty.  
    /// Applies per-channel effects and volume, then records the sample in the
    /// visualiser ring.
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        if !self.playing {
            return 0.0;
        }

        let channel_count = self.effects.len().max(1);
        let channel_index = (self.emitted_samples as usize) % channel_count;

        let raw = match self.queue.pop_front() {
            Some(s) => {
                self.emitted_samples = self.emitted_samples.saturating_add(1);
                self.source_position_samples += self.playback_rate as f64;
                s
            }
            None => {
                if self.stream_finished {
                    self.playing = false;
                    self.status  = PlaybackStatus::Finished;
                } else {
                    // Queue empty but stream not done → underrun.
                    self.underrun_count = self.underrun_count.saturating_add(1);
                }
                return 0.0;
            }
        };

        let mut sample = raw * self.volume;

        // Apply per-channel DSP chain.
        if let Some(effect) = self.effects.get_mut(channel_index) {
            sample = effect.process(sample, channel_index);
        }

        let sample = sample.clamp(-1.0, 1.0);
        self.push_visualizer_sample(sample);
        sample
    }

    // ── Position helpers ──────────────────────────────────────────────────

    /// Current playback position as a `Duration`.
    pub fn position(&self, out_channels: usize) -> Duration {
        if self.sample_rate == 0 || out_channels == 0 {
            return Duration::ZERO;
        }
        let secs = self.source_position_samples
            / (self.sample_rate as f64 * out_channels as f64);
        Duration::from_secs_f64(secs.max(0.0))
    }

    /// Current playback position in milliseconds (for the FFI layer).
    pub fn position_millis(&self, out_channels: usize) -> i32 {
        self.position(out_channels).as_millis() as i32
    }
}