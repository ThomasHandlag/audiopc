use flutter_rust_bridge::frb;

use crate::api::filters::WaveShapeCurve;

/// Soft Clipper
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct SoftClipper {
    drive_val: f32,
    mix_val: f32,
}

impl SoftClipper {
    pub fn new() -> Self { Self { drive_val: 1.0, mix_val: 1.0 } }
    pub fn set_drive(&mut self, v: f32) { self.drive_val = v.max(0.0); }
    pub fn set_mix(&mut self, v: f32) { self.mix_val = v.clamp(0.0, 1.0); }
    pub fn drive(&self) -> f32 { self.drive_val }
    pub fn mix(&self) -> f32 { self.mix_val }

    pub fn process(&mut self, sample: f32) -> f32 {
        let clipped = (sample * self.drive_val).tanh();
        sample * (1.0 - self.mix_val) + clipped * self.mix_val
    }

    pub fn reset(&mut self) {}
}

impl Default for SoftClipper { fn default() -> Self { Self::new() } }

/// Hard Clipper
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct HardClipper {
    threshold_val: f32,
    mix_val: f32,
}

impl HardClipper {
    pub fn new() -> Self { Self { threshold_val: 1.0, mix_val: 1.0 } }
    pub fn set_threshold(&mut self, v: f32) { self.threshold_val = v.max(0.001); }
    pub fn set_mix(&mut self, v: f32) { self.mix_val = v.clamp(0.0, 1.0); }
    pub fn threshold(&self) -> f32 { self.threshold_val }
    pub fn mix(&self) -> f32 { self.mix_val }

    pub fn process(&mut self, sample: f32) -> f32 {
        let clipped = sample.clamp(-self.threshold_val, self.threshold_val);
        sample * (1.0 - self.mix_val) + clipped * self.mix_val
    }

    pub fn reset(&mut self) {}
}

impl Default for HardClipper { fn default() -> Self { Self::new() } }

/// Bit Crusher
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct BitCrusher {
    bit_depth_val: f32,
    rate_reduction_val: f32,
    hold_value: f32,
    counter: f32,
}

impl BitCrusher {
    pub fn new(_sample_rate: f32) -> Self {
        Self { bit_depth_val: 16.0, rate_reduction_val: 1.0, hold_value: 0.0, counter: 0.0 }
    }

    pub fn set_bit_depth(&mut self, v: f32) { self.bit_depth_val = v.clamp(1.0, 32.0); }
    pub fn set_sample_rate_reduction(&mut self, v: f32) { self.rate_reduction_val = v.max(1.0); }
    pub fn bit_depth(&self) -> f32 { self.bit_depth_val }
    pub fn rate_reduction(&self) -> f32 { self.rate_reduction_val }

    pub fn process(&mut self, sample: f32) -> f32 {
        self.counter += 1.0;
        if self.counter >= self.rate_reduction_val {
            self.counter = 0.0;
            self.hold_value = sample;
        }
        let steps = 2.0_f32.powf(self.bit_depth_val);
        (self.hold_value * steps).round() / steps
    }

    pub fn reset(&mut self) { self.hold_value = 0.0; self.counter = 0.0; }
}

/// Waveshaper
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct WaveShaper {
    amount_val: f32,
    curve_type: WaveShapeCurve,
}

impl WaveShaper {
    pub fn new(curve: WaveShapeCurve) -> Self { Self { amount_val: 1.0, curve_type: curve } }
    pub fn set_amount(&mut self, v: f32) { self.amount_val = v; }
    pub fn set_curve(&mut self, v: WaveShapeCurve) { self.curve_type = v; }
    pub fn amount(&self) -> f32 { self.amount_val }
    pub fn curve(&self) -> WaveShapeCurve { self.curve_type }

    fn shape(&self, x: f32) -> f32 {
        let x = x * self.amount_val;
        match self.curve_type {
            WaveShapeCurve::Soft => x.tanh(),
            WaveShapeCurve::Hard => x.clamp(-1.0, 1.0),
            WaveShapeCurve::Asymmetric => if x > 0.0 { (x * 2.0).tanh() * 0.5 } else { (x * 0.5).tanh() * 2.0 },
            WaveShapeCurve::Sine => (x * std::f32::consts::FRAC_PI_2).sin(),
            WaveShapeCurve::Quadratic => x * (1.0 - x.abs()),
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 { self.shape(sample) }
    pub fn reset(&mut self) {}
}

/// Foldback Distortion
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct Foldback {
    threshold_val: f32,
}

impl Foldback {
    pub fn new() -> Self { Self { threshold_val: 1.0 } }
    pub fn set_threshold(&mut self, v: f32) { self.threshold_val = v.max(0.001); }
    pub fn threshold(&self) -> f32 { self.threshold_val }

    pub fn process(&mut self, sample: f32) -> f32 {
        let mut x = sample;
        while x.abs() > self.threshold_val {
            if x > self.threshold_val { x = self.threshold_val - (x - self.threshold_val); }
            else if x < -self.threshold_val { x = -self.threshold_val - (x + self.threshold_val); }
        }
        x
    }

    pub fn reset(&mut self) {}
}

impl Default for Foldback { fn default() -> Self { Self::new() } }