use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, unbounded};
use flutter_rust_bridge::frb;

use crate::{
    api::{
        enums::AudioCommand,
        renderer::{output::AudioOutput, state::PlaybackState},
        source::{AudioSource, decode::DecodePool},
    },
    warn,
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

        let output = Arc::new(Mutex::new(AudioOutput::new(
            tx,
            Arc::clone(&state),
        )));

        AudioPlayer {
            decode,
            output,
            cmd_rc: rc,
            state,
        }
    }

    pub fn set_source(&mut self, source: AudioSource) {
        self.decode.stop();
        self.decode.set_source(source);
        let _ = self.state.lock().map(|mut v| {
            v.clear_audio_state();
            v.stream_ended = false;
            v.pl_state = PlaybackState::Idle;
        });
    }

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

    pub fn is_playing(&self) -> bool {
        self.state
            .lock()
            .map(|v| v.pl_state == PlaybackState::Playing)
            .unwrap_or(false)
    }

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
            s.stream_ended= false;
        }

        let can_play = self.decode.source.is_some() && self.state.lock().map(|v| v.start_millies < self.decode.duration).unwrap_or(false);

        if was_playing || can_play {
            self.play();
        }
    }

    pub fn stop(&mut self) {
        self.decode.stop();
        if let Ok(mut s) = self.state.lock() {
            s.clear_audio_state();
            s.pl_state = PlaybackState::Idle;
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        if let Ok(mut s) = self.state.lock() {
            s.volumn = volume.clamp(0.0, 1.0);
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
