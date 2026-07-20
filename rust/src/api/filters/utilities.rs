use flutter_rust_bridge::frb;

/// Gain
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct Gain {
    gain_linear_val: f32,
    gain_db_val: f32,
}

impl Gain {
    pub fn new() -> Self { Self { gain_linear_val: 1.0, gain_db_val: 0.0 } }
    pub fn from_db(db: f32) -> Self { Self { gain_linear_val: 10.0_f32.powf(db / 20.0), gain_db_val: db } }

    pub fn set_gain_db(&mut self, db: f32) {
        self.gain_db_val = db;
        self.gain_linear_val = 10.0_f32.powf(db / 20.0);
    }

    pub fn set_gain_linear(&mut self, v: f32) {
        self.gain_linear_val = v.max(0.0);
        self.gain_db_val = if v > 0.00001 { 20.0 * v.log10() } else { -100.0 };
    }

    pub fn gain_db(&self) -> f32 { self.gain_db_val }
    pub fn gain_linear(&self) -> f32 { self.gain_linear_val }

    pub fn process(&mut self, sample: f32) -> f32 { sample * self.gain_linear_val }
    pub fn reset(&mut self) {}
}

impl Default for Gain { fn default() -> Self { Self::new() } }

/// DC Remover
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct DCRemover {
    alpha: f32,
    prev_input: f32,
    prev_output: f32,
}

impl DCRemover {
    pub fn new(cutoff_hz: f32, sample_rate: f32) -> Self {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
        let dt = 1.0 / sample_rate;
        Self { alpha: rc / (rc + dt), prev_input: 0.0, prev_output: 0.0 }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let out = self.alpha * (self.prev_output + sample - self.prev_input);
        self.prev_input = sample;
        self.prev_output = out;
        out
    }

    pub fn reset(&mut self) { self.prev_input = 0.0; self.prev_output = 0.0; }
}

/// Panner
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct Panner {
    pan_val: f32,
}

impl Panner {
    pub fn new() -> Self { Self { pan_val: 0.0 } }
    pub fn set_pan(&mut self, v: f32) { self.pan_val = v.clamp(-1.0, 1.0); }
    pub fn pan(&self) -> f32 { self.pan_val }

    pub fn process(&self, sample: f32) -> (f32, f32) {
        ((1.0 - self.pan_val) * 0.5 * sample, (1.0 + self.pan_val) * 0.5 * sample)
    }

    pub fn reset(&mut self) {}
}

impl Default for Panner { fn default() -> Self { Self::new() } }

/// Mute/Solo
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct MuteSolo {
    muted: bool,
    soloed: bool,
    any_solo_active: bool,
}

impl MuteSolo {
    pub fn new() -> Self { Self { muted: false, soloed: false, any_solo_active: false } }
    pub fn set_mute(&mut self, v: bool) { self.muted = v; }
    pub fn set_solo(&mut self, v: bool) { self.soloed = v; }
    pub fn set_any_solo_active(&mut self, v: bool) { self.any_solo_active = v; }
    pub fn is_muted(&self) -> bool { self.muted }
    pub fn is_soloed(&self) -> bool { self.soloed }

    pub fn process(&mut self, sample: f32) -> f32 {
        if self.muted { return 0.0; }
        if self.any_solo_active && !self.soloed { return 0.0; }
        sample
    }

    pub fn reset(&mut self) {}
}

impl Default for MuteSolo { fn default() -> Self { Self::new() } }

/// Sample and Hold
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct SampleAndHold {
    trigger_threshold_val: f32,
    last_value: f32,
    prev_input: f32,
}

impl SampleAndHold {
    pub fn new() -> Self { Self { trigger_threshold_val: 0.0, last_value: 0.0, prev_input: 0.0 } }
    pub fn set_trigger_threshold(&mut self, v: f32) { self.trigger_threshold_val = v; }
    pub fn trigger_threshold(&self) -> f32 { self.trigger_threshold_val }

    pub fn process(&mut self, sample: f32) -> f32 {
        if self.prev_input <= self.trigger_threshold_val && sample > self.trigger_threshold_val {
            self.last_value = sample;
        }
        self.prev_input = sample;
        self.last_value
    }

    pub fn reset(&mut self) { self.last_value = 0.0; self.prev_input = 0.0; }
}

impl Default for SampleAndHold { fn default() -> Self { Self::new() } }

/// Phase Inverter
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct PhaseInverter {
    inverted_val: bool,
}

impl PhaseInverter {
    pub fn new() -> Self { Self { inverted_val: false } }
    pub fn set_inverted(&mut self, v: bool) { self.inverted_val = v; }
    pub fn is_inverted(&self) -> bool { self.inverted_val }

    pub fn process(&mut self, sample: f32) -> f32 { if self.inverted_val { -sample } else { sample } }
    pub fn reset(&mut self) {}
}

impl Default for PhaseInverter { fn default() -> Self { Self::new() } }