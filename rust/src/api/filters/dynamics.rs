use flutter_rust_bridge::frb;

/// Compressor
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct Compressor {
    threshold_val: f32,
    ratio_val: f32,
    attack_val: f32,
    release_val: f32,
    envelope: f32,
    sample_rate: f32,
    makeup_gain_val: f32,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold_val: -20.0, ratio_val: 4.0,
            attack_val: 10.0, release_val: 100.0,
            envelope: 0.0, sample_rate, makeup_gain_val: 0.0,
        }
    }

    pub fn set_threshold(&mut self, v: f32) { self.threshold_val = v; }
    pub fn set_ratio(&mut self, v: f32) { self.ratio_val = v.max(1.0); }
    pub fn set_attack(&mut self, v: f32) { self.attack_val = v.max(0.01); }
    pub fn set_release(&mut self, v: f32) { self.release_val = v.max(0.01); }
    pub fn set_makeup_gain(&mut self, v: f32) { self.makeup_gain_val = v; }

    pub fn threshold(&self) -> f32 { self.threshold_val }
    pub fn ratio(&self) -> f32 { self.ratio_val }
    pub fn attack(&self) -> f32 { self.attack_val }
    pub fn release(&self) -> f32 { self.release_val }
    pub fn makeup_gain(&self) -> f32 { self.makeup_gain_val }

    fn db_to_linear(db: f32) -> f32 { 10.0_f32.powf(db / 20.0) }
    fn linear_to_db(lin: f32) -> f32 { if lin > 0.00001 { 20.0 * lin.log10() } else { -100.0 } }

    pub fn process(&mut self, sample: f32) -> f32 {
        let input_level = sample.abs();
        let env_db = Self::linear_to_db(self.envelope);
        let gr_db = if env_db > self.threshold_val {
            (env_db - self.threshold_val) * (1.0 - 1.0 / self.ratio_val)
        } else { 0.0 };

        let coeff = if input_level > self.envelope {
            (-1.0 / (self.attack_val * self.sample_rate / 1000.0)).exp()
        } else {
            (-1.0 / (self.release_val * self.sample_rate / 1000.0)).exp()
        };
        self.envelope = coeff * self.envelope + (1.0 - coeff) * input_level;

        sample * Self::db_to_linear(-gr_db) * Self::db_to_linear(self.makeup_gain_val)
    }

    pub fn reset(&mut self) { self.envelope = 0.0; }
}

/// Limiter
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct Limiter {
    threshold_val: f32,
    envelope: f32,
    release_val: f32,
    sample_rate: f32,
}

impl Limiter {
    pub fn new(sample_rate: f32) -> Self {
        Self { threshold_val: 1.0, envelope: 0.0, release_val: 50.0, sample_rate }
    }

    pub fn set_threshold(&mut self, v: f32) { self.threshold_val = v.max(0.001); }
    pub fn set_release(&mut self, v: f32) { self.release_val = v.max(0.01); }
    pub fn threshold(&self) -> f32 { self.threshold_val }
    pub fn release(&self) -> f32 { self.release_val }

    pub fn process(&mut self, sample: f32) -> f32 {
        let level = sample.abs();
        self.envelope = if level > self.envelope {
            level
        } else {
            let c = (-1.0 / (self.release_val * self.sample_rate / 1000.0)).exp();
            c * self.envelope + (1.0 - c) * level
        };
        if self.envelope > self.threshold_val { sample * (self.threshold_val / self.envelope) } else { sample }
    }

    pub fn reset(&mut self) { self.envelope = 0.0; }
}

/// Envelope Follower
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct EnvelopeFollower {
    attack_val: f32,
    release_val: f32,
    envelope: f32,
    sample_rate: f32,
}

impl EnvelopeFollower {
    pub fn new(sample_rate: f32) -> Self {
        Self { attack_val: 10.0, release_val: 100.0, envelope: 0.0, sample_rate }
    }

    pub fn set_attack(&mut self, v: f32) { self.attack_val = v.max(0.01); }
    pub fn set_release(&mut self, v: f32) { self.release_val = v.max(0.01); }
    pub fn attack(&self) -> f32 { self.attack_val }
    pub fn release(&self) -> f32 { self.release_val }
    pub fn get_envelope(&self) -> f32 { self.envelope }

    pub fn process(&mut self, sample: f32) -> f32 {
        let level = sample.abs();
        let c = if level > self.envelope {
            (-1.0 / (self.attack_val * self.sample_rate / 1000.0)).exp()
        } else {
            (-1.0 / (self.release_val * self.sample_rate / 1000.0)).exp()
        };
        self.envelope = c * self.envelope + (1.0 - c) * level;
        self.envelope
    }

    pub fn reset(&mut self) { self.envelope = 0.0; }
}

/// Noise Gate
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct NoiseGate {
    threshold_val: f32,
    attack_val: f32,
    hold_val: f32,
    release_val: f32,
    envelope: f32,
    hold_counter: usize,
    sample_rate: f32,
    is_open: bool,
}

impl NoiseGate {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold_val: -60.0, attack_val: 1.0, hold_val: 50.0, release_val: 100.0,
            envelope: 0.0, hold_counter: 0, sample_rate, is_open: false,
        }
    }

    pub fn set_threshold(&mut self, v: f32) { self.threshold_val = v; }
    pub fn set_attack(&mut self, v: f32) { self.attack_val = v.max(0.01); }
    pub fn set_hold(&mut self, v: f32) { self.hold_val = v; }
    pub fn set_release(&mut self, v: f32) { self.release_val = v.max(0.01); }

    pub fn threshold(&self) -> f32 { self.threshold_val }
    pub fn attack(&self) -> f32 { self.attack_val }
    pub fn hold(&self) -> f32 { self.hold_val }
    pub fn release(&self) -> f32 { self.release_val }

    fn db_to_linear(db: f32) -> f32 { 10.0_f32.powf(db / 20.0) }

    pub fn process(&mut self, sample: f32) -> f32 {
        let level = sample.abs();
        let thresh = Self::db_to_linear(self.threshold_val);

        let c = if level > self.envelope {
            (-1.0 / (self.attack_val * self.sample_rate / 1000.0)).exp()
        } else {
            (-1.0 / (self.release_val * self.sample_rate / 1000.0)).exp()
        };
        self.envelope = c * self.envelope + (1.0 - c) * level;

        if self.envelope > thresh {
            self.is_open = true;
            self.hold_counter = (self.hold_val * self.sample_rate / 1000.0) as usize;
        } else if self.hold_counter > 0 {
            self.hold_counter -= 1;
        } else {
            self.is_open = false;
        }
        if self.is_open { sample } else { 0.0 }
    }

    pub fn reset(&mut self) { self.envelope = 0.0; self.hold_counter = 0; self.is_open = false; }
}