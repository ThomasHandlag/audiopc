use flutter_rust_bridge::frb;


/// Simple Delay Line
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
    delay_samples: usize,
    feedback_val: f32,
    mix_val: f32,
}

impl DelayLine {
    pub fn new(max_delay_ms: f32, sample_rate: f32) -> Self {
        let max_samples = (max_delay_ms * sample_rate / 1000.0) as usize;
        Self {
            buffer: vec![0.0; max_samples.max(1)],
            write_pos: 0,
            delay_samples: max_samples / 2,
            feedback_val: 0.0,
            mix_val: 0.5,
        }
    }

    pub fn set_delay_time(&mut self, delay_ms: f32, sample_rate: f32) {
        self.delay_samples = (delay_ms * sample_rate / 1000.0) as usize;
        self.delay_samples = self.delay_samples.clamp(0, self.buffer.len() - 1);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback_val = feedback.clamp(0.0, 0.99);
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix_val = mix.clamp(0.0, 1.0);
    }

    pub fn feedback(&self) -> f32 { self.feedback_val }
    pub fn mix(&self) -> f32 { self.mix_val }

    fn read(&self) -> f32 {
        let read_pos = if self.write_pos >= self.delay_samples {
            self.write_pos - self.delay_samples
        } else {
            self.buffer.len() - (self.delay_samples - self.write_pos)
        };
        self.buffer[read_pos]
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let delayed = self.read();
        self.buffer[self.write_pos] = sample + delayed * self.feedback_val;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        sample * (1.0 - self.mix_val) + delayed * self.mix_val
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

/// Fractional Delay Line
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct FractionalDelay {
    buffer: Vec<f32>,
    write_pos: usize,
    delay_samples: f32,
    feedback_val: f32,
    mix_val: f32,
}

impl FractionalDelay {
    pub fn new(max_delay_ms: f32, sample_rate: f32) -> Self {
        let max_samples = (max_delay_ms * sample_rate / 1000.0) as usize;
        Self {
            buffer: vec![0.0; max_samples.max(2)],
            write_pos: 0,
            delay_samples: max_samples as f32 / 2.0,
            feedback_val: 0.0,
            mix_val: 0.5,
        }
    }

    pub fn set_delay_time(&mut self, delay_ms: f32, sample_rate: f32) {
        self.delay_samples = (delay_ms * sample_rate / 1000.0).clamp(0.0, (self.buffer.len() - 1) as f32);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback_val = feedback.clamp(0.0, 0.99);
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix_val = mix.clamp(0.0, 1.0);
    }

    pub fn feedback(&self) -> f32 { self.feedback_val }
    pub fn mix(&self) -> f32 { self.mix_val }

    fn read_interpolated(&self) -> f32 {
        let delay_int = self.delay_samples.floor() as usize;
        let frac = self.delay_samples - delay_int as f32;

        let pos0 = if self.write_pos >= delay_int {
            self.write_pos - delay_int
        } else {
            self.buffer.len() - (delay_int - self.write_pos)
        };
        let pos1 = if pos0 == 0 { self.buffer.len() - 1 } else { pos0 - 1 };

        self.buffer[pos0] * (1.0 - frac) + self.buffer[pos1] * frac
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let delayed = self.read_interpolated();
        self.buffer[self.write_pos] = sample + delayed * self.feedback_val;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        sample * (1.0 - self.mix_val) + delayed * self.mix_val
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}


/// Revert/Reverse Buffer
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct RevertBuffer {
    record_buffer: Vec<f32>,
    playback_buffer: Vec<f32>,
    record_pos: usize,
    playback_pos: usize,
    length: usize,
    is_recording: bool,
    is_playing: bool,
    mix_val: f32,
    loop_playback: bool,
}

impl RevertBuffer {
    pub fn new(length_ms: f32, sample_rate: f32) -> Self {
        let length = (length_ms * sample_rate / 1000.0) as usize;
        Self {
            record_buffer: vec![0.0; length.max(1)],
            playback_buffer: Vec::new(),
            record_pos: 0, playback_pos: 0,
            length: length.max(1),
            is_recording: true, is_playing: false,
            mix_val: 1.0, loop_playback: false,
        }
    }

    pub fn start_recording(&mut self) {
        self.is_recording = true;
        self.is_playing = false;
        self.record_pos = 0;
        self.playback_pos = 0;
    }

    pub fn start_playback(&mut self) {
        if self.record_pos > 0 {
            self.playback_buffer = self.record_buffer[..self.record_pos].to_vec();
            self.playback_buffer.reverse();
            self.is_recording = false;
            self.is_playing = true;
            self.playback_pos = 0;
        }
    }

    pub fn stop(&mut self) {
        self.is_recording = false;
        self.is_playing = false;
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix_val = mix.clamp(0.0, 1.0);
    }

    pub fn set_loop(&mut self, loop_playback: bool) {
        self.loop_playback = loop_playback;
    }

    pub fn mix(&self) -> f32 { self.mix_val }
    pub fn is_recording(&self) -> bool { self.is_recording }
    pub fn is_playing(&self) -> bool { self.is_playing }

    pub fn process(&mut self, sample: f32) -> f32 {
        let mut output = sample;

        if self.is_recording {
            if self.record_pos < self.length {
                self.record_buffer[self.record_pos] = sample;
                self.record_pos += 1;
            } else {
                self.start_playback();
            }
        }

        if self.is_playing && !self.playback_buffer.is_empty() {
            output = sample * (1.0 - self.mix_val) + self.playback_buffer[self.playback_pos] * self.mix_val;
            self.playback_pos += 1;
            if self.playback_pos >= self.playback_buffer.len() {
                if self.loop_playback { self.playback_pos = 0; } else { self.is_playing = false; }
            }
        }
        output
    }

    pub fn reset(&mut self) {
        self.record_buffer.fill(0.0);
        self.playback_buffer.clear();
        self.record_pos = 0;
        self.playback_pos = 0;
        self.is_recording = true;
        self.is_playing = false;
    }
}