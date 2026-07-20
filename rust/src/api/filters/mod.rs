use flutter_rust_bridge::frb;

use crate::api::filters::delay::{DelayLine, FractionalDelay, RevertBuffer};
use crate::api::filters::distortion::{BitCrusher, Foldback, HardClipper, SoftClipper, WaveShaper};

use crate::api::filters::dynamics::{Compressor, EnvelopeFollower, Limiter, NoiseGate};

use crate::api::filters::utilities::{
    DCRemover, Gain, MuteSolo, Panner, PhaseInverter, SampleAndHold,
};

pub mod delay;
pub mod distortion;
pub mod dynamics;
pub mod utilities;

/// All possible parameter identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Param {
    // Filter params
    Coefficient,
    Frequency,
    Resonance,
    Q,
    GainDb,
    FilterLength,

    // Delay params
    DelayMs,
    Feedback,
    Mix,
    LoopPlayback,

    // Dynamics params
    Threshold,
    Ratio,
    Attack,
    Release,
    Hold,
    MakeupGain,

    // Distortion params
    Drive,
    BitDepth,
    RateReduction,
    Amount,
    CurveType,

    // Utility params
    GainLinear,
    Pan,
    Mute,
    Solo,
    AnySoloActive,
    TriggerThreshold,
    Inverted,

    // Special
    SampleRate,
}

/// Wave shape curve types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WaveShapeCurve {
    Soft,
    Hard,
    Asymmetric,
    Sine,
    Quadratic,
}

/// Biquad filter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiquadType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
    Peaking,
    LowShelf,
    HighShelf,
    AllPass,
}

/// Error type for parameter operations
#[derive(Debug, Clone, PartialEq)]
pub enum ParamError {
    InvalidParam,
    InvalidValue,
}

/// Result of setting a parameter
pub type ParamResult = Result<(), ParamError>;

/// Main processor enum
#[derive(Debug, Clone)]
#[frb(opaque)]
pub enum AudioProcessor {
    // Filters
    OnePoleLowPass(OnePoleLowPass),
    OnePoleHighPass(OnePoleHighPass),
    BiquadFilter(BiquadFilter),
    StateVariableFilter(StateVariableFilter),
    MovingAverage(MovingAverage),

    // Delay
    DelayLine(DelayLine),
    FractionalDelay(FractionalDelay),
    RevertBuffer(RevertBuffer),

    // Dynamics
    Compressor(Compressor),
    Limiter(Limiter),
    EnvelopeFollower(EnvelopeFollower),
    NoiseGate(NoiseGate),

    // Distortion
    SoftClipper(SoftClipper),
    HardClipper(HardClipper),
    BitCrusher(BitCrusher),
    WaveShaper(WaveShaper),
    Foldback(Foldback),

    // Utilities
    Gain(Gain),
    DCRemover(DCRemover),
    Panner(Panner),
    MuteSolo(MuteSolo),
    SampleAndHold(SampleAndHold),
    PhaseInverter(PhaseInverter),

    // Routing
    Chain(Vec<AudioProcessor>),
    Parallel {
        processors: Vec<AudioProcessor>,
        mix: f32,
    },
    Bypass,
}

impl AudioProcessor {
    /// Process a single sample
    pub fn process(&mut self, sample: f32) -> f32 {
        match self {
            AudioProcessor::OnePoleLowPass(p) => p.process(sample),
            AudioProcessor::OnePoleHighPass(p) => p.process(sample),
            AudioProcessor::BiquadFilter(p) => p.process(sample),
            AudioProcessor::StateVariableFilter(p) => p.process(sample),
            AudioProcessor::MovingAverage(p) => p.process(sample),

            AudioProcessor::DelayLine(p) => p.process(sample),
            AudioProcessor::FractionalDelay(p) => p.process(sample),
            AudioProcessor::RevertBuffer(p) => p.process(sample),

            AudioProcessor::Compressor(p) => p.process(sample),
            AudioProcessor::Limiter(p) => p.process(sample),
            AudioProcessor::EnvelopeFollower(p) => p.process(sample),
            AudioProcessor::NoiseGate(p) => p.process(sample),

            AudioProcessor::SoftClipper(p) => p.process(sample),
            AudioProcessor::HardClipper(p) => p.process(sample),
            AudioProcessor::BitCrusher(p) => p.process(sample),
            AudioProcessor::WaveShaper(p) => p.process(sample),
            AudioProcessor::Foldback(p) => p.process(sample),

            AudioProcessor::Gain(p) => p.process(sample),
            AudioProcessor::DCRemover(p) => p.process(sample),
            AudioProcessor::Panner(p) => p.process(sample).0,
            AudioProcessor::MuteSolo(p) => p.process(sample),
            AudioProcessor::SampleAndHold(p) => p.process(sample),
            AudioProcessor::PhaseInverter(p) => p.process(sample),

            AudioProcessor::Chain(processors) => processors.iter_mut().fold(sample, |s, p| p.process(s)),
            AudioProcessor::Parallel { processors, mix } => {
                if processors.is_empty() {
                    return sample;
                }
                let wet: f32 = processors
                    .iter_mut()
                    .map(|p| p.process(sample))
                    .sum::<f32>()
                    / processors.len() as f32;
                sample * (1.0 as f32 - *mix) + wet * (*mix)
            }
            AudioProcessor::Bypass => sample,
        }
    }

    /// Reset processor state
    pub fn reset(&mut self) {
        match self {
            AudioProcessor::OnePoleLowPass(p) => p.reset(),
            AudioProcessor::OnePoleHighPass(p) => p.reset(),
            AudioProcessor::BiquadFilter(p) => p.reset(),
            AudioProcessor::StateVariableFilter(p) => p.reset(),
            AudioProcessor::MovingAverage(p) => p.reset(),

            AudioProcessor::DelayLine(p) => p.reset(),
            AudioProcessor::FractionalDelay(p) => p.reset(),
            AudioProcessor::RevertBuffer(p) => p.reset(),

            AudioProcessor::Compressor(p) => p.reset(),
            AudioProcessor::Limiter(p) => p.reset(),
            AudioProcessor::EnvelopeFollower(p) => p.reset(),
            AudioProcessor::NoiseGate(p) => p.reset(),

            AudioProcessor::SoftClipper(p) => p.reset(),
            AudioProcessor::HardClipper(p) => p.reset(),
            AudioProcessor::BitCrusher(p) => p.reset(),
            AudioProcessor::WaveShaper(p) => p.reset(),
            AudioProcessor::Foldback(p) => p.reset(),

            AudioProcessor::Gain(p) => p.reset(),
            AudioProcessor::DCRemover(p) => p.reset(),
            AudioProcessor::Panner(p) => p.reset(),
            AudioProcessor::MuteSolo(p) => p.reset(),
            AudioProcessor::SampleAndHold(p) => p.reset(),
            AudioProcessor::PhaseInverter(p) => p.reset(),

            AudioProcessor::Chain(processors) => processors.iter_mut().for_each(|p| p.reset()),
            AudioProcessor::Parallel { processors, .. } => processors.iter_mut().for_each(|p| p.reset()),
            AudioProcessor::Bypass => {}
        }
    }

    /// Get processor name
    pub const fn name(&self) -> &'static str {
        match self {
            AudioProcessor::OnePoleLowPass(_) => "OnePoleLowPass",
            AudioProcessor::OnePoleHighPass(_) => "OnePoleHighPass",
            AudioProcessor::BiquadFilter(_) => "BiquadFilter",
            AudioProcessor::StateVariableFilter(_) => "StateVariableFilter",
            AudioProcessor::MovingAverage(_) => "MovingAverage",
            AudioProcessor::DelayLine(_) => "DelayLine",
            AudioProcessor::FractionalDelay(_) => "FractionalDelay",
            AudioProcessor::RevertBuffer(_) => "RevertBuffer",
            AudioProcessor::Compressor(_) => "Compressor",
            AudioProcessor::Limiter(_) => "Limiter",
            AudioProcessor::EnvelopeFollower(_) => "EnvelopeFollower",
            AudioProcessor::NoiseGate(_) => "NoiseGate",
            AudioProcessor::SoftClipper(_) => "SoftClipper",
            AudioProcessor::HardClipper(_) => "HardClipper",
            AudioProcessor::BitCrusher(_) => "BitCrusher",
            AudioProcessor::WaveShaper(_) => "WaveShaper",
            AudioProcessor::Foldback(_) => "Foldback",
            AudioProcessor::Gain(_) => "Gain",
            AudioProcessor::DCRemover(_) => "DCRemover",
            AudioProcessor::Panner(_) => "Panner",
            AudioProcessor::MuteSolo(_) => "MuteSolo",
            AudioProcessor::SampleAndHold(_) => "SampleAndHold",
            AudioProcessor::PhaseInverter(_) => "PhaseInverter",
            AudioProcessor::Chain(_) => "Chain",
            AudioProcessor::Parallel { .. } => "Parallel",
            AudioProcessor::Bypass => "Bypass",
        }
    }

    /// Set a float parameter
    pub fn set(&mut self, param: Param, value: f32) -> ParamResult {
        match self {
            AudioProcessor::OnePoleLowPass(p) => match param {
                Param::Coefficient => {
                    p.set_coefficient(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::OnePoleHighPass(p) => match param {
                Param::Coefficient => {
                    p.set_coefficient(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::BiquadFilter(p) => match param {
                Param::Frequency => {
                    p.set_frequency(value);
                    Ok(())
                }
                Param::Q => {
                    p.set_q(value);
                    Ok(())
                }
                Param::GainDb => {
                    p.set_gain(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::StateVariableFilter(p) => match param {
                Param::Frequency => {
                    p.set_frequency(value);
                    Ok(())
                }
                Param::Resonance => {
                    p.set_resonance(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::DelayLine(p) => match param {
                Param::DelayMs => {
                    p.set_delay_time(value, 44100.0);
                    Ok(())
                }
                Param::Feedback => {
                    p.set_feedback(value);
                    Ok(())
                }
                Param::Mix => {
                    p.set_mix(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::FractionalDelay(p) => match param {
                Param::DelayMs => {
                    p.set_delay_time(value, 44100.0);
                    Ok(())
                }
                Param::Feedback => {
                    p.set_feedback(value);
                    Ok(())
                }
                Param::Mix => {
                    p.set_mix(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::RevertBuffer(p) => match param {
                Param::Mix => {
                    p.set_mix(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::Compressor(p) => match param {
                Param::Threshold => {
                    p.set_threshold(value);
                    Ok(())
                }
                Param::Ratio => {
                    p.set_ratio(value);
                    Ok(())
                }
                Param::Attack => {
                    p.set_attack(value);
                    Ok(())
                }
                Param::Release => {
                    p.set_release(value);
                    Ok(())
                }
                Param::MakeupGain => {
                    p.set_makeup_gain(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::Limiter(p) => match param {
                Param::Threshold => {
                    p.set_threshold(value);
                    Ok(())
                }
                Param::Release => {
                    p.set_release(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::EnvelopeFollower(p) => match param {
                Param::Attack => {
                    p.set_attack(value);
                    Ok(())
                }
                Param::Release => {
                    p.set_release(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::NoiseGate(p) => match param {
                Param::Threshold => {
                    p.set_threshold(value);
                    Ok(())
                }
                Param::Attack => {
                    p.set_attack(value);
                    Ok(())
                }
                Param::Hold => {
                    p.set_hold(value);
                    Ok(())
                }
                Param::Release => {
                    p.set_release(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::SoftClipper(p) => match param {
                Param::Drive => {
                    p.set_drive(value);
                    Ok(())
                }
                Param::Mix => {
                    p.set_mix(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::HardClipper(p) => match param {
                Param::Threshold => {
                    p.set_threshold(value);
                    Ok(())
                }
                Param::Mix => {
                    p.set_mix(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::BitCrusher(p) => match param {
                Param::BitDepth => {
                    p.set_bit_depth(value);
                    Ok(())
                }
                Param::RateReduction => {
                    p.set_sample_rate_reduction(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::WaveShaper(p) => match param {
                Param::Amount => {
                    p.set_amount(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::Foldback(p) => match param {
                Param::Threshold => {
                    p.set_threshold(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::Gain(p) => match param {
                Param::GainDb => {
                    p.set_gain_db(value);
                    Ok(())
                }
                Param::GainLinear => {
                    p.set_gain_linear(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::Panner(p) => match param {
                Param::Pan => {
                    p.set_pan(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::MuteSolo(p) => match param {
                Param::Mute => {
                    p.set_mute(value > 0.5);
                    Ok(())
                }
                Param::Solo => {
                    p.set_solo(value > 0.5);
                    Ok(())
                }
                Param::AnySoloActive => {
                    p.set_any_solo_active(value > 0.5);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::SampleAndHold(p) => match param {
                Param::TriggerThreshold => {
                    p.set_trigger_threshold(value);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::PhaseInverter(p) => match param {
                Param::Inverted => {
                    p.set_inverted(value > 0.5);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::Parallel { mix, .. } => match param {
                Param::Mix => {
                    *mix = value.clamp(0.0, 1.0);
                    Ok(())
                }
                _ => Err(ParamError::InvalidParam),
            },
            _ => Err(ParamError::InvalidParam),
        }
    }

    /// Get a float parameter value
    pub fn get(&self, param: Param) -> Result<f32, ParamError> {
        match self {
            AudioProcessor::OnePoleLowPass(p) => match param {
                Param::Coefficient => Ok(p.coefficient()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::OnePoleHighPass(p) => match param {
                Param::Coefficient => Ok(p.coefficient()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::BiquadFilter(p) => match param {
                Param::Frequency => Ok(p.frequency()),
                Param::Q => Ok(p.q()),
                Param::GainDb => Ok(p.gain_db()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::StateVariableFilter(p) => match param {
                Param::Frequency => Ok(p.frequency()),
                Param::Resonance => Ok(p.resonance()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::DelayLine(p) => match param {
                Param::Feedback => Ok(p.feedback()),
                Param::Mix => Ok(p.mix()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::FractionalDelay(p) => match param {
                Param::Feedback => Ok(p.feedback()),
                Param::Mix => Ok(p.mix()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::RevertBuffer(p) => match param {
                Param::Mix => Ok(p.mix()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::Compressor(p) => match param {
                Param::Threshold => Ok(p.threshold()),
                Param::Ratio => Ok(p.ratio()),
                Param::Attack => Ok(p.attack()),
                Param::Release => Ok(p.release()),
                Param::MakeupGain => Ok(p.makeup_gain()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::Limiter(p) => match param {
                Param::Threshold => Ok(p.threshold()),
                Param::Release => Ok(p.release()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::EnvelopeFollower(p) => match param {
                Param::Attack => Ok(p.attack()),
                Param::Release => Ok(p.release()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::NoiseGate(p) => match param {
                Param::Threshold => Ok(p.threshold()),
                Param::Attack => Ok(p.attack()),
                Param::Hold => Ok(p.hold()),
                Param::Release => Ok(p.release()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::SoftClipper(p) => match param {
                Param::Drive => Ok(p.drive()),
                Param::Mix => Ok(p.mix()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::HardClipper(p) => match param {
                Param::Threshold => Ok(p.threshold()),
                Param::Mix => Ok(p.mix()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::BitCrusher(p) => match param {
                Param::BitDepth => Ok(p.bit_depth()),
                Param::RateReduction => Ok(p.rate_reduction()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::WaveShaper(p) => match param {
                Param::Amount => Ok(p.amount()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::Foldback(p) => match param {
                Param::Threshold => Ok(p.threshold()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::Gain(p) => match param {
                Param::GainDb => Ok(p.gain_db()),
                Param::GainLinear => Ok(p.gain_linear()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::Panner(p) => match param {
                Param::Pan => Ok(p.pan()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::MuteSolo(p) => match param {
                Param::Mute => Ok(if p.is_muted() { 1.0 } else { 0.0 }),
                Param::Solo => Ok(if p.is_soloed() { 1.0 } else { 0.0 }),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::SampleAndHold(p) => match param {
                Param::TriggerThreshold => Ok(p.trigger_threshold()),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::PhaseInverter(p) => match param {
                Param::Inverted => Ok(if p.is_inverted() { 1.0 } else { 0.0 }),
                _ => Err(ParamError::InvalidParam),
            },
            AudioProcessor::Parallel { mix, .. } => match param {
                Param::Mix => Ok(*mix),
                _ => Err(ParamError::InvalidParam),
            },
            _ => Err(ParamError::InvalidParam),
        }
    }

    /// Get list of valid params for this processor
    pub fn valid_params(&self) -> &'static [Param] {
        match self {
            AudioProcessor::OnePoleLowPass(_) | AudioProcessor::OnePoleHighPass(_) => &[Param::Coefficient],
            AudioProcessor::BiquadFilter(_) => &[Param::Frequency, Param::Q, Param::GainDb],
            AudioProcessor::StateVariableFilter(_) => &[Param::Frequency, Param::Resonance],
            AudioProcessor::DelayLine(_) | AudioProcessor::FractionalDelay(_) => {
                &[Param::DelayMs, Param::Feedback, Param::Mix]
            }
            AudioProcessor::RevertBuffer(_) => &[Param::Mix],
            AudioProcessor::Compressor(_) => &[
                Param::Threshold,
                Param::Ratio,
                Param::Attack,
                Param::Release,
                Param::MakeupGain,
            ],
            AudioProcessor::Limiter(_) => &[Param::Threshold, Param::Release],
            AudioProcessor::EnvelopeFollower(_) => &[Param::Attack, Param::Release],
            AudioProcessor::NoiseGate(_) => {
                &[Param::Threshold, Param::Attack, Param::Hold, Param::Release]
            }
            AudioProcessor::SoftClipper(_) => &[Param::Drive, Param::Mix],
            AudioProcessor::HardClipper(_) => &[Param::Threshold, Param::Mix],
            AudioProcessor::BitCrusher(_) => &[Param::BitDepth, Param::RateReduction],
            AudioProcessor::WaveShaper(_) => &[Param::Amount],
            AudioProcessor::Foldback(_) => &[Param::Threshold],
            AudioProcessor::Gain(_) => &[Param::GainDb, Param::GainLinear],
            AudioProcessor::Panner(_) => &[Param::Pan],
            AudioProcessor::MuteSolo(_) => &[Param::Mute, Param::Solo, Param::AnySoloActive],
            AudioProcessor::SampleAndHold(_) => &[Param::TriggerThreshold],
            AudioProcessor::PhaseInverter(_) => &[Param::Inverted],
            AudioProcessor::Parallel { .. } => &[Param::Mix],
            AudioProcessor::Chain(_)
            | AudioProcessor::Bypass
            | AudioProcessor::MovingAverage(_)
            | AudioProcessor::DCRemover(_) => &[],
        }
    }
}

impl Default for AudioProcessor {
    fn default() -> Self {
        AudioProcessor::Bypass
    }
}

impl std::fmt::Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Param::Coefficient => write!(f, "Coefficient"),
            Param::Frequency => write!(f, "Frequency"),
            Param::Resonance => write!(f, "Resonance"),
            Param::Q => write!(f, "Q"),
            Param::GainDb => write!(f, "GainDb"),
            Param::FilterLength => write!(f, "FilterLength"),
            Param::DelayMs => write!(f, "DelayMs"),
            Param::Feedback => write!(f, "Feedback"),
            Param::Mix => write!(f, "Mix"),
            Param::LoopPlayback => write!(f, "LoopPlayback"),
            Param::Threshold => write!(f, "Threshold"),
            Param::Ratio => write!(f, "Ratio"),
            Param::Attack => write!(f, "Attack"),
            Param::Release => write!(f, "Release"),
            Param::Hold => write!(f, "Hold"),
            Param::MakeupGain => write!(f, "MakeupGain"),
            Param::Drive => write!(f, "Drive"),
            Param::BitDepth => write!(f, "BitDepth"),
            Param::RateReduction => write!(f, "RateReduction"),
            Param::Amount => write!(f, "Amount"),
            Param::CurveType => write!(f, "CurveType"),
            Param::GainLinear => write!(f, "GainLinear"),
            Param::Pan => write!(f, "Pan"),
            Param::Mute => write!(f, "Mute"),
            Param::Solo => write!(f, "Solo"),
            Param::AnySoloActive => write!(f, "AnySoloActive"),
            Param::TriggerThreshold => write!(f, "TriggerThreshold"),
            Param::Inverted => write!(f, "Inverted"),
            Param::SampleRate => write!(f, "SampleRate"),
        }
    }
}

/// One-Pole Low-Pass Filter
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct OnePoleLowPass {
    coefficient: f32,
    state: f32,
}

impl OnePoleLowPass {
    pub fn new() -> Self {
        Self {
            coefficient: 0.5,
            state: 0.0,
        }
    }

    pub fn from_cutoff(cutoff: f32, sample_rate: f32) -> Self {
        let cutoff = cutoff.clamp(0.0, 0.99);
        let coefficient = 1.0 - (2.0 * std::f32::consts::PI * cutoff * sample_rate).exp();
        Self {
            coefficient,
            state: 0.0,
        }
    }

    pub fn set_coefficient(&mut self, coeff: f32) {
        self.coefficient = coeff.clamp(0.0, 1.0);
    }

    pub fn coefficient(&self) -> f32 {
        self.coefficient
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        self.state += self.coefficient * (sample - self.state);
        self.state
    }

    pub fn reset(&mut self) {
        self.state = 0.0;
    }
}

impl Default for OnePoleLowPass {
    fn default() -> Self {
        Self::new()
    }
}

/// One-Pole High-Pass Filter
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct OnePoleHighPass {
    lowpass: OnePoleLowPass,
    prev_input: f32,
}

impl OnePoleHighPass {
    pub fn new() -> Self {
        Self {
            lowpass: OnePoleLowPass::new(),
            prev_input: 0.0,
        }
    }

    pub fn from_cutoff(cutoff: f32, sample_rate: f32) -> Self {
        Self {
            lowpass: OnePoleLowPass::from_cutoff(cutoff, sample_rate),
            prev_input: 0.0,
        }
    }

    pub fn set_coefficient(&mut self, coeff: f32) {
        self.lowpass.set_coefficient(coeff);
    }

    pub fn coefficient(&self) -> f32 {
        self.lowpass.coefficient()
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let lowpass_out = self.lowpass.process(sample);
        let output = sample - lowpass_out;
        self.prev_input = sample;
        output
    }

    pub fn reset(&mut self) {
        self.lowpass.reset();
        self.prev_input = 0.0;
    }
}

impl Default for OnePoleHighPass {
    fn default() -> Self {
        Self::new()
    }
}

/// Biquad Filter
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct BiquadFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    filter_type: BiquadType,
    sample_rate: f32,
    frequency: f32,
    q: f32,
    gain_db: f32,
}

impl BiquadFilter {
    pub fn new(filter_type: BiquadType, sample_rate: f32) -> Self {
        let mut filter = Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            filter_type,
            sample_rate,
            frequency: 1000.0,
            q: 0.707,
            gain_db: 0.0,
        };
        filter.calculate_coefficients();
        filter
    }

    fn calculate_coefficients(&mut self) {
        let w0 = 2.0 * std::f32::consts::PI * self.frequency / self.sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * self.q);
        let a = 10.0_f32.powf(self.gain_db / 40.0);

        let (b0, b1, b2, a0, a1, a2) = match self.filter_type {
            BiquadType::LowPass => {
                let b0 = (1.0 - cos_w0) / 2.0;
                (
                    b0,
                    1.0 - cos_w0,
                    b0,
                    1.0 + alpha,
                    -2.0 * cos_w0,
                    1.0 - alpha,
                )
            }
            BiquadType::HighPass => {
                let b0 = (1.0 + cos_w0) / 2.0;
                (
                    b0,
                    -(1.0 + cos_w0),
                    b0,
                    1.0 + alpha,
                    -2.0 * cos_w0,
                    1.0 - alpha,
                )
            }
            BiquadType::BandPass => (alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha),
            BiquadType::Notch => (
                1.0,
                -2.0 * cos_w0,
                1.0,
                1.0 + alpha,
                -2.0 * cos_w0,
                1.0 - alpha,
            ),
            BiquadType::Peaking => (
                1.0 + alpha * a,
                -2.0 * cos_w0,
                1.0 - alpha * a,
                1.0 + alpha / a,
                -2.0 * cos_w0,
                1.0 - alpha / a,
            ),
            BiquadType::LowShelf => {
                let sq = 2.0 * a.sqrt() * alpha;
                (
                    a * ((a + 1.0) - (a - 1.0) * cos_w0 + sq),
                    2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0),
                    a * ((a + 1.0) - (a - 1.0) * cos_w0 - sq),
                    (a + 1.0) + (a - 1.0) * cos_w0 + sq,
                    -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0),
                    (a + 1.0) + (a - 1.0) * cos_w0 - sq,
                )
            }
            BiquadType::HighShelf => {
                let sq = 2.0 * a.sqrt() * alpha;
                (
                    a * ((a + 1.0) + (a - 1.0) * cos_w0 + sq),
                    -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0),
                    a * ((a + 1.0) + (a - 1.0) * cos_w0 - sq),
                    (a + 1.0) - (a - 1.0) * cos_w0 + sq,
                    2.0 * ((a - 1.0) - (a + 1.0) * cos_w0),
                    (a + 1.0) - (a - 1.0) * cos_w0 - sq,
                )
            }
            BiquadType::AllPass => (
                1.0 - alpha,
                -2.0 * cos_w0,
                1.0 + alpha,
                1.0 + alpha,
                -2.0 * cos_w0,
                1.0 - alpha,
            ),
        };

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.max(1.0).min(self.sample_rate / 2.0);
        self.calculate_coefficients();
    }

    pub fn set_q(&mut self, q: f32) {
        self.q = q.max(0.001);
        self.calculate_coefficients();
    }

    pub fn set_gain(&mut self, gain_db: f32) {
        self.gain_db = gain_db;
        self.calculate_coefficients();
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }
    pub fn q(&self) -> f32 {
        self.q
    }
    pub fn gain_db(&self) -> f32 {
        self.gain_db
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let output = self.b0 * sample + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = sample;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

/// State Variable Filter outputs
#[derive(Debug, Clone, Copy)]
#[frb(opaque)]
pub struct SVFOutputs {
    pub low: f32,
    pub high: f32,
    pub band: f32,
    pub notch: f32,
}

/// State Variable Filter
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct StateVariableFilter {
    frequency: f32,
    resonance: f32,
    low: f32,
    band: f32,
    high: f32,
    notch: f32,
    sample_rate: f32,
}

impl StateVariableFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            frequency: 1000.0,
            resonance: 0.5,
            low: 0.0,
            band: 0.0,
            high: 0.0,
            notch: 0.0,
            sample_rate,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(20.0, self.sample_rate / 2.0);
    }

    pub fn set_resonance(&mut self, res: f32) {
        self.resonance = res.clamp(0.0, 1.0);
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }
    pub fn resonance(&self) -> f32 {
        self.resonance
    }

    pub fn process_all(&mut self, sample: f32) -> SVFOutputs {
        let f = 2.0 * std::f32::consts::PI * self.frequency / self.sample_rate;
        let q = 1.0 / (1.0 - self.resonance * 0.99);
        let g = (f / 2.0).tan();
        let k = 1.0 / q;

        let hp = (sample - (self.low + g * self.band + k * self.band)) / (1.0 + g * (g + k));
        let bp = self.band + g * hp;
        let lp = self.low + g * bp;

        self.low = lp;
        self.band = bp;
        self.high = hp;
        self.notch = hp + lp;

        SVFOutputs {
            low: lp,
            high: hp,
            band: bp,
            notch: hp + lp,
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        self.process_all(sample).low
    }

    pub fn reset(&mut self) {
        self.low = 0.0;
        self.band = 0.0;
        self.high = 0.0;
        self.notch = 0.0;
    }
}

/// Moving Average Filter
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct MovingAverage {
    buffer: Vec<f32>,
    index: usize,
    sum: f32,
}

impl MovingAverage {
    pub fn new(length: usize) -> Self {
        Self {
            buffer: vec![0.0; length.max(1)],
            index: 0,
            sum: 0.0,
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        self.sum -= self.buffer[self.index];
        self.buffer[self.index] = sample;
        self.sum += sample;
        self.index = (self.index + 1) % self.buffer.len();
        self.sum / self.buffer.len() as f32
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
        self.sum = 0.0;
    }
}
