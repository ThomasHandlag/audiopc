use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, unbounded};
use flutter_rust_bridge::frb;

use crate::{
    api::{
        enums::AudioCommand, filters::AudioProcessor, renderer::{output::{AudioOuputConfig, AudioOutput}, state::PlaybackState}, source::{AudioSource, decode::DecodePool}
    }, warn,
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
    cmd_rc: Receiver<AudioCommand>,
}

impl AudioPlayer {
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

        let state = Arc::new(Mutex::new(AudioState::new(0, config, v_config)));

        let output = Arc::new(Mutex::new(AudioOutput::new(tx, Arc::clone(&state))));

        AudioPlayer {
            decode,
            output,
            cmd_rc: rc,
            state,
        }
    }

    /// Set source for preparing to play.
    #[frb(sync)]
    pub fn set_source(&mut self, source: AudioSource) {
        self.decode.stop();
        self.decode.set_source(source);
        let _ = self.state.lock().map(|mut v| {
            v.clear_audio_state();
            v.stream_ended = false;
            v.pl_state = PlaybackState::Idle;
        });
    }

    /// Play current audio source. If current source is empty then an error will be throw.
    #[frb(sync)]
    pub fn play(&mut self) {
        let rc = self.cmd_rc.clone();

        let _ = self
            .state
            .lock()
            .map(|mut v| v.pl_state = PlaybackState::Playing);

        let state = Arc::clone(&self.state);

        let output = Arc::clone(&self.output);

        let _ = self.output.lock().map(|mut v| v.build());

        self.decode.build(
            move || {
                while let Ok(cmd) = rc.try_recv() {
                    match cmd {
                        AudioCommand::RebuildOuput => {
                            let _ = output.lock().map(|mut v| v.build());
                        }
                    }
                }
            },
            state,
            AudioOutput::get_output_config(),
        );
    }

    /// Pause curent playing song.
    #[frb(sync)]
    pub fn pause(&mut self) {
        if let Ok(mut s) = self
            .state
            .lock() {
                s.pl_state = PlaybackState::Paused
            }
    }

    #[frb(sync)]
    pub fn is_completed(&self) -> bool {
        self.state.lock().map(|v| v.pl_state == PlaybackState::Completed).unwrap_or(false)
    }

    /// Whether player is playing or not.
    #[frb(sync)]
    pub fn is_playing(&self) -> bool {
        self.state
            .lock()
            .map(|v| v.pl_state == PlaybackState::Playing)
            .unwrap_or(false)
    }

    /// Play audio at given position in millisecond.
    pub fn seek(&mut self, position: i32) {
        let mut target = position.max(0);
        if self.decode.duration > 0 {
            target = target.min(self.decode.duration);
        }

        if !matches!(self.decode.source, Some(_)) {
            warn!("Seek called with no source");
            return;
        }

        let was_playing = self.is_playing();

        self.decode.stop();
        let _ = self.state.lock().map(|mut v| v.start_millies = target);

        if let Ok(mut s) = self.state.lock() {
            s.queue.clear();
            s.visualizer.clear();
            s.emitted_samples = 0;
            s.stream_ended = false;
        }

        let can_play = self.decode.source.is_some()
            && self
                .state
                .lock()
                .map(|v| v.start_millies < self.decode.duration)
                .unwrap_or(false);

        if was_playing || can_play {
            self.play();
        }
    }

    /// Stop current playing audio.
    pub fn stop(&mut self) {
        self.decode.stop();
        if let Ok(mut s) = self.state.lock() {
            s.clear_audio_state();
            s.pl_state = PlaybackState::Stopped;
        }
    }

    /// Set volume of ouput audio.
    pub fn set_volumn(&mut self, volumn: f32) {
        if let Ok(mut s) = self.state.lock() {
            s.volumn = volumn.clamp(0.0, 1.0);
        }
    }

    /// Set playback speed of output audio.
    pub fn set_rate(&mut self, rate: f32) {
        if let Ok(mut s) = self.state.lock() {
            s.pl_rate = rate.clamp(0.2, 3.0)
        }
    }

    /// Add Effect to output audio.
    pub fn add_effect(&mut self, effect: AudioProcessor) {
        if let Ok(mut s) = self.state.lock() {
           s.effects.push(effect);
        }
    }

    /// Duration in millisecond of current source.
    #[frb(sync)]
    pub fn duration_millis(&self) -> i32 {
        self.decode.duration
    }

    /// Clear the effect chain of every output channel.
    pub fn clear_effects(&mut self) {
        if let Ok(mut s) = self.state.lock() {
            s.effects.clear();
        }
    }

    pub fn position(&self) -> i32 {
        self.state.lock().map(|v| v.start_millies).unwrap_or(-1)
    }

    pub fn get_state(&self) -> i32 {
        self.state.lock().map(|v| v.pl_state.id()).unwrap_or(1)
    }

    pub fn samples_data(&self) -> Vec<f32> {
        self.state.lock().map(|v| v.visualizer.iter().copied().collect::<Vec<f32>>()).unwrap_or(vec![])
    }

    #[frb(sync)]
    pub fn get_output_config(&self) -> AudioOuputConfig {
        AudioOutput::get_output_config()
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
