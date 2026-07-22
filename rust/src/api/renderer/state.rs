use std::{collections::VecDeque, time::Duration};

use crate::api::{
    filters::AudioProcessor, renderer::{state::PlaybackState::{Idle, Playing}},
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
    pub(crate) source_position: f64,
    pub(crate) queue: VecDeque<f32>,
    pub(crate) visualizer: VecDeque<f32>,
    pub(crate) volumn: f32,
    pub(crate) emitted_samples: u64,
    pub(crate) underrun_count: u32,
    pub(crate) effects: Vec<AudioProcessor>,
    pub(crate) stream_ended: bool,
    config: BuffConfig,
    v_config: BuffConfig,
    channels: usize
}

impl AudioState {
    pub fn new(config: BuffConfig, v_config: BuffConfig, channels: usize) -> AudioState {
        AudioState {
            pl_rate: 1.0,
            pl_state: Idle,
            source_position: 0.0,
            effects: vec![],
            underrun_count: 0,
            emitted_samples: 0,
            volumn: 1.0,
            queue: VecDeque::with_capacity(config.max_samples),
            visualizer: VecDeque::with_capacity(v_config.max_samples),
            stream_ended: false,
            config,
            v_config,
            channels
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
                self.source_position += self.pl_rate as f64;
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

        let mut sample = raw * self.volumn;

        let channel_index = (self.emitted_samples as usize) % channels;

        if let Some(effect) = self.effects.get_mut(channel_index) {
            sample = effect.process(sample);
        }


        let sample = sample.clamp(-1.0, 1.0);
        self.push_visualizer_sample(sample);
        sample
    }

    /// Current playback position in milliseconds.
    pub fn position(&self) -> i32 {
        if self.config.sample_rate == 0 {
            return 0;
        }
        let secs =
            self.source_position as f64 / (self.config.sample_rate as f64 * self.channels as f64);
        Duration::from_secs_f64(secs.max(0.0)).as_millis() as i32
    }

    /// Clear all transient audio state without touching volume / rate / device.
    pub fn clear_audio_state(&mut self) {
        self.emitted_samples = 0;
        self.source_position = 0.0;
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
