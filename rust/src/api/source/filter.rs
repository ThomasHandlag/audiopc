use flutter_rust_bridge::frb;

/// Main processor enum - all processors in one type
#[derive(Debug, Clone)]
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
    Parallel { processors: Vec<AudioProcessor>, mix: f32 },
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
            AudioProcessor::Panner(p) => p.process(sample).0, // Return left channel
            AudioProcessor::MuteSolo(p) => p.process(sample),
            AudioProcessor::SampleAndHold(p) => p.process(sample),
            AudioProcessor::PhaseInverter(p) => p.process(sample),
            
            AudioProcessor::Chain(processors) => {
                let mut output = sample;
                for p in processors.iter_mut() {
                    output = p.process(output);
                }
                output
            }
            AudioProcessor::Parallel { processors, mix } => {
                if processors.is_empty() {
                    return sample;
                }
                let wet_sum: f32 = processors
                    .iter_mut()
                    .map(|p| p.process(sample))
                    .sum::<f32>() / processors.len() as f32;
                sample * (1.0 - *mix) + wet_sum * *mix
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
            
            AudioProcessor::Chain(processors) => {
                for p in processors.iter_mut() {
                    p.reset();
                }
            }
            AudioProcessor::Parallel { processors, .. } => {
                for p in processors.iter_mut() {
                    p.reset();
                }
            }
            AudioProcessor::Bypass => {}
        }
    }

    /// Get processor name as string
    pub fn name(&self) -> &'static str {
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

    /// Set a parameter by name (returns false if param not found)
    pub fn set_param(&mut self, name: &str, value: f32) -> bool {
        match self {
            AudioProcessor::OnePoleLowPass(p) => match name {
                "coefficient" => { p.set_coefficient(value); true }
                _ => false,
            }
            AudioProcessor::OnePoleHighPass(p) => match name {
                "coefficient" => { p.set_coefficient(value); true }
                _ => false,
            }
            AudioProcessor::BiquadFilter(p) => match name {
                "frequency" => { p.set_frequency(value); true }
                "q" => { p.set_q(value); true }
                "gain_db" => { p.set_gain(value); true }
                _ => false,
            }
            AudioProcessor::StateVariableFilter(p) => match name {
                "frequency" => { p.set_frequency(value); true }
                "resonance" => { p.set_resonance(value); true }
                _ => false,
            }
            AudioProcessor::DelayLine(p) => match name {
                "delay_ms" => { p.set_delay_time(value, 44100.0); true }
                "feedback" => { p.set_feedback(value); true }
                "mix" => { p.set_mix(value); true }
                _ => false,
            }
            AudioProcessor::FractionalDelay(p) => match name {
                "delay_ms" => { p.set_delay_time(value, 44100.0); true }
                "feedback" => { p.set_feedback(value); true }
                "mix" => { p.set_mix(value); true }
                _ => false,
            }
            AudioProcessor::RevertBuffer(p) => match name {
                "mix" => { p.set_mix(value); true }
                _ => false,
            }
            AudioProcessor::Compressor(p) => match name {
                "threshold" => { p.set_threshold(value); true }
                "ratio" => { p.set_ratio(value); true }
                "attack" => { p.set_attack(value); true }
                "release" => { p.set_release(value); true }
                "makeup_gain" => { p.set_makeup_gain(value); true }
                _ => false,
            }
            AudioProcessor::Limiter(p) => match name {
                "threshold" => { p.set_threshold(value); true }
                "release" => { p.set_release(value); true }
                _ => false,
            }
            AudioProcessor::EnvelopeFollower(p) => match name {
                "attack" => { p.set_attack(value); true }
                "release" => { p.set_release(value); true }
                _ => false,
            }
            AudioProcessor::NoiseGate(p) => match name {
                "threshold" => { p.set_threshold(value); true }
                "attack" => { p.set_attack(value); true }
                "hold" => { p.set_hold(value); true }
                "release" => { p.set_release(value); true }
                _ => false,
            }
            AudioProcessor::SoftClipper(p) => match name {
                "drive" => { p.set_drive(value); true }
                "mix" => { p.set_mix(value); true }
                _ => false,
            }
            AudioProcessor::HardClipper(p) => match name {
                "threshold" => { p.set_threshold(value); true }
                "mix" => { p.set_mix(value); true }
                _ => false,
            }
            AudioProcessor::BitCrusher(p) => match name {
                "bit_depth" => { p.set_bit_depth(value); true }
                "rate_reduction" => { p.set_sample_rate_reduction(value); true }
                _ => false,
            }
            AudioProcessor::WaveShaper(p) => match name {
                "amount" => { p.set_amount(value); true }
                _ => false,
            }
            AudioProcessor::Foldback(p) => match name {
                "threshold" => { p.set_threshold(value); true }
                _ => false,
            }
            AudioProcessor::Gain(p) => match name {
                "gain_db" => { p.set_gain_db(value); true }
                "gain_linear" => { p.set_gain_linear(value); true }
                _ => false,
            }
            AudioProcessor::Panner(p) => match name {
                "pan" => { p.set_pan(value); true }
                _ => false,
            }
            AudioProcessor::MuteSolo(p) => match name {
                "mute" => { p.set_mute(value > 0.5); true }
                "solo" => { p.set_solo(value > 0.5); true }
                _ => false,
            }
            AudioProcessor::SampleAndHold(p) => match name {
                "trigger_threshold" => { p.set_trigger_threshold(value); true }
                _ => false,
            }
            AudioProcessor::PhaseInverter(p) => match name {
                "inverted" => { p.set_inverted(value > 0.5); true }
                _ => false,
            }
            AudioProcessor::Parallel { mix, .. } => {
                if name == "mix" {
                    *mix = value.clamp(0.0, 1.0);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl Default for AudioProcessor {
    fn default() -> Self {
        AudioProcessor::Bypass
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
        Self { coefficient, state: 0.0 }
    }

    pub fn set_coefficient(&mut self, coeff: f32) {
        self.coefficient = coeff.clamp(0.0, 1.0);
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

/// Biquad Filter Types
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Biquad Filter (Second-Order IIR)
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
            b0: 1.0, b1: 0.0, b2: 0.0,
            a1: 0.0, a2: 0.0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
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
                let b1 = 1.0 - cos_w0;
                let b2 = (1.0 - cos_w0) / 2.0;
                (b0, b1, b2, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
            }
            BiquadType::HighPass => {
                let b0 = (1.0 + cos_w0) / 2.0;
                let b1 = -(1.0 + cos_w0);
                let b2 = (1.0 + cos_w0) / 2.0;
                (b0, b1, b2, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
            }
            BiquadType::BandPass => {
                (alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
            }
            BiquadType::Notch => {
                (1.0, -2.0 * cos_w0, 1.0, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
            }
            BiquadType::Peaking => {
                let b0 = 1.0 + alpha * a;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 - alpha * a;
                let a0 = 1.0 + alpha / a;
                (b0, b1, b2, a0, -2.0 * cos_w0, 1.0 - alpha / a)
            }
            BiquadType::LowShelf => {
                let sq = 2.0 * a.sqrt() * alpha;
                let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + sq);
                let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
                let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - sq);
                let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + sq;
                let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
                let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - sq;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::HighShelf => {
                let sq = 2.0 * a.sqrt() * alpha;
                let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + sq);
                let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
                let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - sq);
                let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + sq;
                let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
                let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - sq;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::AllPass => {
                let b0 = 1.0 - alpha;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 + alpha;
                (b0, b1, b2, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
            }
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

    pub fn process(&mut self, sample: f32) -> f32 {
        let output = self.b0 * sample
            + self.b1 * self.x1
            + self.b2 * self.x2
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

/// State Variable Filter (ZDF)
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

    pub fn process_all(&mut self, sample: f32) -> SVFOutputs {
        let f = 2.0 * std::f32::consts::PI * self.frequency / self.sample_rate;
        let q = 1.0 / (1.0 - self.resonance * 0.99);

        let g = (f / 2.0).tan();
        let k = 1.0 / q;

        let hp = (sample - (self.low + g * self.band + k * self.band)) / (1.0 + g * (g + k));
        let bp = self.band + g * hp;
        let lp = self.low + g * bp;
        let notch = hp + lp;

        self.low = lp;
        self.band = bp;
        self.high = hp;
        self.notch = notch;

        SVFOutputs { low: lp, high: hp, band: bp, notch }
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

/// Moving Average Filter (FIR)
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

/// Simple Delay Line
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
    delay_samples: usize,
    feedback: f32,
    mix: f32,
}

impl DelayLine {
    pub fn new(max_delay_ms: f32, sample_rate: f32) -> Self {
        let max_samples = (max_delay_ms * sample_rate / 1000.0) as usize;
        Self {
            buffer: vec![0.0; max_samples.max(1)],
            write_pos: 0,
            delay_samples: max_samples / 2,
            feedback: 0.0,
            mix: 0.5,
        }
    }

    pub fn set_delay_time(&mut self, delay_ms: f32, sample_rate: f32) {
        self.delay_samples = (delay_ms * sample_rate / 1000.0) as usize;
        self.delay_samples = self.delay_samples.clamp(0, self.buffer.len() - 1);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.99);
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

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
        self.buffer[self.write_pos] = sample + delayed * self.feedback;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        sample * (1.0 - self.mix) + delayed * self.mix
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

/// Fractional Delay Line with Linear Interpolation
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct FractionalDelay {
    buffer: Vec<f32>,
    write_pos: usize,
    delay_samples: f32,
    feedback: f32,
    mix: f32,
}

impl FractionalDelay {
    pub fn new(max_delay_ms: f32, sample_rate: f32) -> Self {
        let max_samples = (max_delay_ms * sample_rate / 1000.0) as usize;
        Self {
            buffer: vec![0.0; max_samples.max(2)],
            write_pos: 0,
            delay_samples: max_samples as f32 / 2.0,
            feedback: 0.0,
            mix: 0.5,
        }
    }

    pub fn set_delay_time(&mut self, delay_ms: f32, sample_rate: f32) {
        self.delay_samples = (delay_ms * sample_rate / 1000.0)
            .clamp(0.0, (self.buffer.len() - 1) as f32);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.99);
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    fn read_interpolated(&self) -> f32 {
        let delay_int = self.delay_samples.floor() as usize;
        let frac = self.delay_samples - delay_int as f32;

        let pos0 = if self.write_pos >= delay_int {
            self.write_pos - delay_int
        } else {
            self.buffer.len() - (delay_int - self.write_pos)
        };

        let pos1 = if pos0 == 0 {
            self.buffer.len() - 1
        } else {
            pos0 - 1
        };

        self.buffer[pos0] * (1.0 - frac) + self.buffer[pos1] * frac
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let delayed = self.read_interpolated();
        self.buffer[self.write_pos] = sample + delayed * self.feedback;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        sample * (1.0 - self.mix) + delayed * self.mix
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
    mix: f32,
    loop_playback: bool,
}

impl RevertBuffer {
    pub fn new(length_ms: f32, sample_rate: f32) -> Self {
        let length = (length_ms * sample_rate / 1000.0) as usize;
        Self {
            record_buffer: vec![0.0; length.max(1)],
            playback_buffer: Vec::new(),
            record_pos: 0,
            playback_pos: 0,
            length: length.max(1),
            is_recording: true,
            is_playing: false,
            mix: 1.0,
            loop_playback: false,
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
        self.mix = mix.clamp(0.0, 1.0);
    }

    pub fn set_loop(&mut self, loop_playback: bool) {
        self.loop_playback = loop_playback;
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

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
            output = sample * (1.0 - self.mix) + self.playback_buffer[self.playback_pos] * self.mix;
            self.playback_pos += 1;

            if self.playback_pos >= self.playback_buffer.len() {
                if self.loop_playback {
                    self.playback_pos = 0;
                } else {
                    self.is_playing = false;
                }
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

/// Compressor
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct Compressor {
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    envelope: f32,
    sample_rate: f32,
    makeup_gain: f32,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold: -20.0,
            ratio: 4.0,
            attack: 10.0,
            release: 100.0,
            envelope: 0.0,
            sample_rate,
            makeup_gain: 0.0,
        }
    }

    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.threshold = threshold_db;
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.max(1.0);
    }

    pub fn set_attack(&mut self, attack_ms: f32) {
        self.attack = attack_ms.max(0.01);
    }

    pub fn set_release(&mut self, release_ms: f32) {
        self.release = release_ms.max(0.01);
    }

    pub fn set_makeup_gain(&mut self, gain_db: f32) {
        self.makeup_gain = gain_db;
    }

    fn db_to_linear(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    fn linear_to_db(linear: f32) -> f32 {
        if linear > 0.00001 {
            20.0 * linear.log10()
        } else {
            -100.0
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let input_level = sample.abs();
        let env_db = Self::linear_to_db(self.envelope);

        let gain_reduction_db = if env_db > self.threshold {
            (env_db - self.threshold) * (1.0 - 1.0 / self.ratio)
        } else {
            0.0
        };

        let target_env = input_level;
        let coeff = if input_level > self.envelope {
            (-1.0 / (self.attack * self.sample_rate / 1000.0)).exp()
        } else {
            (-1.0 / (self.release * self.sample_rate / 1000.0)).exp()
        };

        self.envelope = coeff * self.envelope + (1.0 - coeff) * target_env;

        let gain_reduction = Self::db_to_linear(-gain_reduction_db);
        let makeup = Self::db_to_linear(self.makeup_gain);

        sample * gain_reduction * makeup
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }
}

/// Limiter
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct Limiter {
    threshold: f32,
    envelope: f32,
    release: f32,
    sample_rate: f32,
}

impl Limiter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold: 1.0,
            envelope: 0.0,
            release: 50.0,
            sample_rate,
        }
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.max(0.001);
    }

    pub fn set_release(&mut self, release_ms: f32) {
        self.release = release_ms.max(0.01);
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let input_level = sample.abs();

        if input_level > self.envelope {
            self.envelope = input_level;
        } else {
            let coeff = (-1.0 / (self.release * self.sample_rate / 1000.0)).exp();
            self.envelope = coeff * self.envelope + (1.0 - coeff) * input_level;
        }

        if self.envelope > self.threshold {
            sample * (self.threshold / self.envelope)
        } else {
            sample
        }
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }
}

/// Envelope Follower
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct EnvelopeFollower {
    attack: f32,
    release: f32,
    envelope: f32,
    sample_rate: f32,
}

impl EnvelopeFollower {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            attack: 10.0,
            release: 100.0,
            envelope: 0.0,
            sample_rate,
        }
    }

    pub fn set_attack(&mut self, attack_ms: f32) {
        self.attack = attack_ms.max(0.01);
    }

    pub fn set_release(&mut self, release_ms: f32) {
        self.release = release_ms.max(0.01);
    }

    pub fn get_envelope(&self) -> f32 {
        self.envelope
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let input_level = sample.abs();

        let coeff = if input_level > self.envelope {
            (-1.0 / (self.attack * self.sample_rate / 1000.0)).exp()
        } else {
            (-1.0 / (self.release * self.sample_rate / 1000.0)).exp()
        };

        self.envelope = coeff * self.envelope + (1.0 - coeff) * input_level;
        self.envelope
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }
}

/// Noise Gate
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct NoiseGate {
    threshold: f32,
    attack: f32,
    hold: f32,
    release: f32,
    envelope: f32,
    hold_counter: usize,
    sample_rate: f32,
    is_open: bool,
}

impl NoiseGate {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold: -60.0,
            attack: 1.0,
            hold: 50.0,
            release: 100.0,
            envelope: 0.0,
            hold_counter: 0,
            sample_rate,
            is_open: false,
        }
    }

    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.threshold = threshold_db;
    }

    pub fn set_attack(&mut self, attack_ms: f32) {
        self.attack = attack_ms.max(0.01);
    }

    pub fn set_hold(&mut self, hold_ms: f32) {
        self.hold = hold_ms;
    }

    pub fn set_release(&mut self, release_ms: f32) {
        self.release = release_ms.max(0.01);
    }

    fn db_to_linear(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let input_level = sample.abs();
        let threshold_linear = Self::db_to_linear(self.threshold);

        let coeff = if input_level > self.envelope {
            (-1.0 / (self.attack * self.sample_rate / 1000.0)).exp()
        } else {
            (-1.0 / (self.release * self.sample_rate / 1000.0)).exp()
        };
        self.envelope = coeff * self.envelope + (1.0 - coeff) * input_level;

        if self.envelope > threshold_linear {
            self.is_open = true;
            self.hold_counter = (self.hold * self.sample_rate / 1000.0) as usize;
        } else if self.hold_counter > 0 {
            self.hold_counter -= 1;
        } else {
            self.is_open = false;
        }

        if self.is_open { sample } else { 0.0 }
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.hold_counter = 0;
        self.is_open = false;
    }
}


/// Soft Clipper (tanh)
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct SoftClipper {
    drive: f32,
    mix: f32,
}

impl SoftClipper {
    pub fn new() -> Self {
        Self { drive: 1.0, mix: 1.0 }
    }

    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.max(0.0);
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let driven = sample * self.drive;
        let clipped = driven.tanh();
        sample * (1.0 - self.mix) + clipped * self.mix
    }

    pub fn reset(&mut self) {}
}

impl Default for SoftClipper {
    fn default() -> Self {
        Self::new()
    }
}

/// Hard Clipper
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct HardClipper {
    threshold: f32,
    mix: f32,
}

impl HardClipper {
    pub fn new() -> Self {
        Self { threshold: 1.0, mix: 1.0 }
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.max(0.001);
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let clipped = sample.clamp(-self.threshold, self.threshold);
        sample * (1.0 - self.mix) + clipped * self.mix
    }

    pub fn reset(&mut self) {}
}

impl Default for HardClipper {
    fn default() -> Self {
        Self::new()
    }
}

/// Bit Crusher
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct BitCrusher {
    bit_depth: f32,
    sample_rate_reduction: f32,
    hold_value: f32,
    counter: f32,
}

impl BitCrusher {
    pub fn new(_sample_rate: f32) -> Self {
        Self {
            bit_depth: 16.0,
            sample_rate_reduction: 1.0,
            hold_value: 0.0,
            counter: 0.0,
        }
    }

    pub fn set_bit_depth(&mut self, bits: f32) {
        self.bit_depth = bits.clamp(1.0, 32.0);
    }

    pub fn set_sample_rate_reduction(&mut self, factor: f32) {
        self.sample_rate_reduction = factor.max(1.0);
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        self.counter += 1.0;
        if self.counter >= self.sample_rate_reduction {
            self.counter = 0.0;
            self.hold_value = sample;
        }

        let held = self.hold_value;
        let steps = (2.0_f32).powf(self.bit_depth);
        (held * steps).round() / steps
    }

    pub fn reset(&mut self) {
        self.hold_value = 0.0;
        self.counter = 0.0;
    }
}

/// Waveshaper curve types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaveShapeCurve {
    Soft,
    Hard,
    Asymmetric,
    Sine,
    Quadratic,
}

/// Waveshaper
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct WaveShaper {
    amount: f32,
    curve_type: WaveShapeCurve,
}

impl WaveShaper {
    pub fn new(curve_type: WaveShapeCurve) -> Self {
        Self { amount: 1.0, curve_type }
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount;
    }

    pub fn set_curve(&mut self, curve: WaveShapeCurve) {
        self.curve_type = curve;
    }

    fn shape(&self, x: f32) -> f32 {
        let x = x * self.amount;
        match self.curve_type {
            WaveShapeCurve::Soft => x.tanh(),
            WaveShapeCurve::Hard => x.clamp(-1.0, 1.0),
            WaveShapeCurve::Asymmetric => {
                if x > 0.0 {
                    (x * 2.0).tanh() * 0.5
                } else {
                    (x * 0.5).tanh() * 2.0
                }
            }
            WaveShapeCurve::Sine => (x * std::f32::consts::FRAC_PI_2).sin(),
            WaveShapeCurve::Quadratic => x * (1.0 - x.abs()),
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        self.shape(sample)
    }

    pub fn reset(&mut self) {}
}

/// Foldback Distortion
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct Foldback {
    threshold: f32,
}

impl Foldback {
    pub fn new() -> Self {
        Self { threshold: 1.0 }
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.max(0.001);
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let mut x = sample;
        let threshold = self.threshold;

        while x.abs() > threshold {
            if x > threshold {
                x = threshold - (x - threshold);
            } else if x < -threshold {
                x = -threshold - (x + threshold);
            }
        }
        x
    }

    pub fn reset(&mut self) {}
}

impl Default for Foldback {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple Gain
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct Gain {
    gain_linear: f32,
    gain_db: f32,
}

impl Gain {
    pub fn new() -> Self {
        Self { gain_linear: 1.0, gain_db: 0.0 }
    }

    pub fn from_db(db: f32) -> Self {
        Self {
            gain_linear: 10.0_f32.powf(db / 20.0),
            gain_db: db,
        }
    }

    pub fn set_gain_db(&mut self, db: f32) {
        self.gain_db = db;
        self.gain_linear = 10.0_f32.powf(db / 20.0);
    }

    pub fn set_gain_linear(&mut self, gain: f32) {
        self.gain_linear = gain.max(0.0);
        self.gain_db = if gain > 0.00001 {
            20.0 * gain.log10()
        } else {
            -100.0
        };
    }

    pub fn get_gain_db(&self) -> f32 {
        self.gain_db
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        sample * self.gain_linear
    }

    pub fn reset(&mut self) {}
}

impl Default for Gain {
    fn default() -> Self {
        Self::new()
    }
}

/// DC Offset Removal
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
        let alpha = rc / (rc + dt);

        Self { alpha, prev_input: 0.0, prev_output: 0.0 }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let output = self.alpha * (self.prev_output + sample - self.prev_input);
        self.prev_input = sample;
        self.prev_output = output;
        output
    }

    pub fn reset(&mut self) {
        self.prev_input = 0.0;
        self.prev_output = 0.0;
    }
}

/// Panner
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct Panner {
    pan: f32,
}

impl Panner {
    pub fn new() -> Self {
        Self { pan: 0.0 }
    }

    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    pub fn process(&self, sample: f32) -> (f32, f32) {
        let left_gain = (1.0 - self.pan) * 0.5;
        let right_gain = (1.0 + self.pan) * 0.5;
        (sample * left_gain, sample * right_gain)
    }

    pub fn reset(&mut self) {}
}

impl Default for Panner {
    fn default() -> Self {
        Self::new()
    }
}

/// Mute/Solo Switch
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct MuteSolo {
    muted: bool,
    soloed: bool,
    any_solo_active: bool,
}

impl MuteSolo {
    pub fn new() -> Self {
        Self { muted: false, soloed: false, any_solo_active: false }
    }

    pub fn set_mute(&mut self, muted: bool) {
        self.muted = muted;
    }

    pub fn set_solo(&mut self, soloed: bool) {
        self.soloed = soloed;
    }

    pub fn set_any_solo_active(&mut self, active: bool) {
        self.any_solo_active = active;
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        if self.muted { return 0.0; }
        if self.any_solo_active && !self.soloed { return 0.0; }
        sample
    }

    pub fn reset(&mut self) {}
}

impl Default for MuteSolo {
    fn default() -> Self {
        Self::new()
    }
}

/// Sample and Hold
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct SampleAndHold {
    trigger_threshold: f32,
    last_value: f32,
    prev_input: f32,
}

impl SampleAndHold {
    pub fn new() -> Self {
        Self { trigger_threshold: 0.0, last_value: 0.0, prev_input: 0.0 }
    }

    pub fn set_trigger_threshold(&mut self, threshold: f32) {
        self.trigger_threshold = threshold;
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        if self.prev_input <= self.trigger_threshold && sample > self.trigger_threshold {
            self.last_value = sample;
        }
        self.prev_input = sample;
        self.last_value
    }

    pub fn reset(&mut self) {
        self.last_value = 0.0;
        self.prev_input = 0.0;
    }
}

impl Default for SampleAndHold {
    fn default() -> Self {
        Self::new()
    }
}

/// Phase Inverter
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct PhaseInverter {
    inverted: bool,
}

impl PhaseInverter {
    pub fn new() -> Self {
        Self { inverted: false }
    }

    pub fn set_inverted(&mut self, inverted: bool) {
        self.inverted = inverted;
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        if self.inverted { -sample } else { sample }
    }

    pub fn reset(&mut self) {}
}

impl Default for PhaseInverter {
    fn default() -> Self {
        Self::new()
    }
}