use std::{collections::VecDeque, time::Duration};

use crate::api::{
    renderer::state::PlaybackState::{Idle, Playing},
    source::filter::AudioProcessor,
};
use flutter_rust_bridge::frb;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Idle,
    Buffering,
    Playing,
    Paused,
    Stopped,
    Completed,
    Error(String),
}

/// Carries fractional position and boundary samples across decode packets.
///
/// Linear interpolation is used to convert between the source sample rate and
/// the device's native rate (and playback speed).  This state prevents audible
/// glitches at packet boundaries.
#[frb(opaque)]
pub struct ResampleState {
    /// Current fractional position within the current packet (source frames).
    pub(crate) pos: f64,
    /// The last *source* frame of the previous packet, used as the left
    /// neighbour for the first interpolated output sample of the next packet.
    pub(crate) carry: Vec<f32>,
}

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

#[frb(opaque)]
pub struct AudioState {
    pub(crate) pl_state: PlaybackState,
    pub(crate) pl_rate: f32,
    pub(crate) start_millies: i32,
    pub(crate) queue: VecDeque<f32>,
    pub(crate) visualizer: VecDeque<f32>,
    pub(crate) volumn: f32,
    pub(crate) emitted_samples: u64,
    pub(crate) underrun_count: u32,
    pub(crate) effects: Vec<AudioProcessor>,
    pub(crate) stream_ended: bool,
    config: BuffConfig,
    v_config: BuffConfig,
}

impl AudioState {
    pub fn new(start_millies: i32, config: BuffConfig, v_config: BuffConfig) -> AudioState {
        AudioState {
            pl_rate: 1.0,
            pl_state: Idle,
            start_millies,
            effects: vec![],
            underrun_count: 0,
            emitted_samples: 0,
            volumn: 1.0,
            queue: VecDeque::with_capacity(config.max_samples),
            visualizer: VecDeque::with_capacity(v_config.max_samples),
            stream_ended: false,
            config,
            v_config,
        }
    }

    #[inline]
    pub fn next_sample(&mut self, channels: usize) -> f32 {
        if self.pl_state != Playing {
            return 0.0;
        }

        let raw = match self.queue.pop_front() {
            Some(s) => {
                self.emitted_samples = self.emitted_samples.saturating_add(1);
                let mut source_pos = self.compute_source(channels as u16);
                source_pos += self.pl_rate as f64;
                self.start_millies += self.compute_millies(source_pos, channels);
                s
            }
            None => {
                if self.stream_ended {
                    self.pl_state = PlaybackState::Completed;
                } else {
                    // Queue empty but stream not done → underrun.
                    self.underrun_count = self.underrun_count.saturating_add(1);
                }
                return 0.0;
            }
        };

        let sample = raw * self.volumn;

        let sample = sample.clamp(-1.0, 1.0);
        self.push_visualizer_sample(sample);
        sample
    }

    pub fn compute_source(&self, channels: u16) -> f64 {
        let target = self.start_millies;
        let target_samples = ((target as u64)
            .saturating_mul(self.config.sample_rate as u64)
            .saturating_mul(channels as u64)
            / 1000) as f64;
        target_samples
    }

    pub fn compute_millies(&self, source_pos: f64, channels: usize) -> i32 {
        let secs =
            source_pos / (self.config.sample_rate as f64 * channels as f64);

        Duration::from_secs_f64(secs.max(0.0)).as_millis() as i32
    }

    /// Current playback position in milliseconds.
    pub fn position(&self, out_channels: usize) -> i32 {
        if self.config.sample_rate == 0 || out_channels == 0 {
            return 0;
        }
        let secs =
            self.start_millies as f64 / (self.config.sample_rate as f64 * out_channels as f64);
        Duration::from_secs_f64(secs.max(0.0)).as_millis() as i32
    }

    /// Clear all transient audio state without touching volume / rate / device.
    pub fn clear_audio_state(&mut self) {
        self.emitted_samples = 0;
        self.start_millies = 0;
        self.stream_ended = true;
        self.underrun_count = 0;
        self.effects = vec![];
    }

    pub fn push_visualizer_sample(&mut self, sample: f32) {
        if self.v_config.max_samples == 0 {
            return;
        }
        if self.visualizer.len() >= self.v_config.max_samples {
            self.visualizer.pop_front();
        }
        self.visualizer.push_back(sample);
    }

    /// Push up to `max_samples - queue.len()` samples.  Returns the count
    /// actually pushed; caller retries the rest after sleeping.
    pub fn push_samples_bounded(&mut self, samples: &[f32]) -> usize {
        let available = self.config.max_samples.saturating_sub(self.queue.len());
        let push_count = samples.len().min(available);
        self.queue.extend(samples.iter().take(push_count).copied());
        push_count
    }
}
