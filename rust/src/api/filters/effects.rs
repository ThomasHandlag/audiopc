use std::f32::consts::PI;

use flutter_rust_bridge::frb;

// ═══════════════════════════════════════════════════════════════
//  Gain
// ═══════════════════════════════════════════════════════════════

/// Simple linear gain stage.
///
/// `gain_db` of 0.0 → unity; negative values attenuate, positive amplify.
#[frb(opaque)]
#[derive(Clone)]
pub struct GainEffect {
    pub gain_db: f32,
    linear: f32,
    pub bypass: bool,
}

impl GainEffect {
    pub fn new(gain_db: f32) -> Self {
        let linear = db_to_linear(gain_db);
        Self { gain_db, linear, bypass: false }
    }

    pub fn set_gain_db(&mut self, gain_db: f32) {
        self.gain_db = gain_db;
        self.linear = db_to_linear(gain_db);
    }

    #[inline]
    pub fn process(&mut self, sample: f32) -> f32 {
        if self.bypass { sample } else { sample * self.linear }
    }
}

// ═══════════════════════════════════════════════════════════════
//  Soft-clip distortion / overdrive
// ═══════════════════════════════════════════════════════════════

/// Soft-clip overdrive using a tanh waveshaper.
///
/// `drive`  – input gain before clipping (1.0 = clean, higher = more distortion).
/// `output` – compensating output level (linear).
#[frb(opaque)]
#[derive(Clone)]
pub struct OverdriveEffect {
    pub drive: f32,
    pub output: f32,
    pub bypass: bool,
}

impl OverdriveEffect {
    pub fn new(drive: f32, output: f32) -> Self {
        Self { drive: drive.max(1.0), output, bypass: false }
    }

    #[inline]
    pub fn process(&mut self, sample: f32) -> f32 {
        if self.bypass { return sample; }
        (sample * self.drive).tanh() * self.output
    }
}

// ═══════════════════════════════════════════════════════════════
//  Hard-clip / Bitcrusher-style distortion
// ═══════════════════════════════════════════════════════════════

/// Hard-clip distortion with a symmetrical threshold in [-1, 1].
#[frb(opaque)]
#[derive(Clone)]
pub struct HardClipEffect {
    /// Clip threshold (0 < threshold ≤ 1.0).
    pub threshold: f32,
    pub bypass: bool,
}

impl HardClipEffect {
    pub fn new(threshold: f32) -> Self {
        Self { threshold: threshold.clamp(0.001, 1.0), bypass: false }
    }

    #[inline]
    pub fn process(&mut self, sample: f32) -> f32 {
        if self.bypass { return sample; }
        sample.clamp(-self.threshold, self.threshold) / self.threshold
    }
}

// ═══════════════════════════════════════════════════════════════
//  Dynamics: Compressor / Limiter
// ═══════════════════════════════════════════════════════════════

/// Feed-forward RMS compressor / limiter.
///
/// Works in log domain for accurate dB gain computation.
#[frb(opaque)]
#[derive(Clone)]
pub struct CompressorEffect {
    /// Threshold in dB above which compression kicks in.
    pub threshold_db: f32,
    /// Compression ratio (e.g. 4.0 = 4:1). Use very high values for limiting.
    pub ratio: f32,
    /// Attack time in seconds.
    pub attack_s: f32,
    /// Release time in seconds.
    pub release_s: f32,
    /// Make-up gain in dB.
    pub makeup_db: f32,
    pub bypass: bool,

    // Internal state
    sample_rate: f32,
    envelope: f32, // current dB envelope
}

impl CompressorEffect {
    pub fn new(
        threshold_db: f32,
        ratio: f32,
        attack_s: f32,
        release_s: f32,
        makeup_db: f32,
        sample_rate: f32,
    ) -> Self {
        Self {
            threshold_db,
            ratio: ratio.max(1.0),
            attack_s,
            release_s,
            makeup_db,
            bypass: false,
            sample_rate,
            envelope: threshold_db, // start at threshold to avoid initial pumping
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn reset(&mut self) {
        self.envelope = self.threshold_db;
    }

    #[inline]
    pub fn process(&mut self, sample: f32) -> f32 {
        if self.bypass { return sample; }

        // Detect: convert sample amplitude to dB
        let input_db = linear_to_db(sample.abs().max(1e-9));

        // Gain-computer
        let over = input_db - self.threshold_db;
        let target_db = if over > 0.0 {
            self.threshold_db + over / self.ratio
        } else {
            input_db
        };

        // Level detector with attack / release
        let coeff = if target_db > self.envelope {
            time_to_coeff(self.attack_s, self.sample_rate)
        } else {
            time_to_coeff(self.release_s, self.sample_rate)
        };
        self.envelope = self.envelope * coeff + target_db * (1.0 - coeff);

        // Gain reduction
        let gain_db = (self.envelope - input_db) + self.makeup_db;
        sample * db_to_linear(gain_db)
    }
}

// ═══════════════════════════════════════════════════════════════
//  Delay / Echo
// ═══════════════════════════════════════════════════════════════

/// Single-tap stereo delay (mono-in, mono-out).
///
/// Produces a single echo with optional feedback.
#[frb(opaque)]
#[derive(Clone)]
pub struct DelayEffect {
    /// Mix between dry (0.0) and wet (1.0).
    pub wet: f32,
    /// Feedback fraction (0.0 – <1.0).
    pub feedback: f32,
    pub bypass: bool,

    buffer: Vec<f32>,
    write: usize,
    delay_samples: usize,
}

impl DelayEffect {
    pub fn new(delay_seconds: f32, feedback: f32, wet: f32, sample_rate: f32) -> Self {
        let max_samples = (sample_rate * 4.0) as usize; // up to 4 s
        let delay_samples = ((delay_seconds * sample_rate) as usize).clamp(1, max_samples);
        Self {
            wet: wet.clamp(0.0, 1.0),
            feedback: feedback.clamp(0.0, 0.98),
            bypass: false,
            buffer: vec![0.0; max_samples],
            write: 0,
            delay_samples,
        }
    }

    /// Change delay time without clearing the buffer.
    pub fn set_delay(&mut self, delay_seconds: f32, sample_rate: f32) {
        let max = self.buffer.len();
        self.delay_samples = ((delay_seconds * sample_rate) as usize).clamp(1, max);
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
    }

    #[inline]
    pub fn process(&mut self, sample: f32) -> f32 {
        if self.bypass { return sample; }

        let buf_len = self.buffer.len();
        let read = (self.write + buf_len - self.delay_samples) % buf_len;
        let delayed = self.buffer[read];

        self.buffer[self.write] = sample + delayed * self.feedback;
        self.write = (self.write + 1) % buf_len;

        sample * (1.0 - self.wet) + delayed * self.wet
    }
}

// ═══════════════════════════════════════════════════════════════
//  Chorus
// ═══════════════════════════════════════════════════════════════

/// Multi-voice chorus with LFO-modulated delay.
///
/// Uses 2 voices; for richer chorus add more.
#[frb(opaque)]
#[derive(Clone)]
pub struct ChorusEffect {
    /// LFO frequency in Hz.
    pub rate: f32,
    /// Maximum modulation depth in seconds.
    pub depth: f32,
    /// Dry/wet mix (0.0 = dry, 1.0 = fully wet).
    pub wet: f32,
    pub bypass: bool,

    sample_rate: f32,
    buffer: Vec<f32>,
    write: usize,
    lfo_phase: [f32; 2],
    lfo_offset: [f32; 2], // phase offset per voice
}

impl ChorusEffect {
    /// * `rate`  – LFO Hz (typical 0.1–5.0)
    /// * `depth` – modulation depth in seconds (typical 0.002–0.030)
    /// * `wet`   – mix (0–1)
    pub fn new(rate: f32, depth: f32, wet: f32, sample_rate: f32) -> Self {
        let max_buf = (sample_rate * 0.1) as usize; // 100 ms max
        Self {
            rate,
            depth: depth.max(0.0),
            wet: wet.clamp(0.0, 1.0),
            bypass: false,
            sample_rate,
            buffer: vec![0.0; max_buf],
            write: 0,
            lfo_phase: [0.0, 0.0],
            lfo_offset: [0.0, PI],
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.lfo_phase = [0.0, 0.0];
    }

    #[inline]
    pub fn process(&mut self, sample: f32) -> f32 {
        if self.bypass { return sample; }

        let buf_len = self.buffer.len();
        self.buffer[self.write] = sample;

        let lfo_inc = 2.0 * PI * self.rate / self.sample_rate;
        let depth_samples = self.depth * self.sample_rate;

        let mut wet_sum = 0.0_f32;
        for i in 0..2 {
            self.lfo_phase[i] = (self.lfo_phase[i] + lfo_inc) % (2.0 * PI);
            let phase = self.lfo_phase[i] + self.lfo_offset[i];
            let mod_samples = depth_samples * (phase.sin() * 0.5 + 0.5) + 1.0;

            // Linear interpolation between two adjacent samples
            let delay_f = mod_samples;
            let delay_i = delay_f as usize;
            let frac = delay_f - delay_i as f32;

            let r0 = (self.write + buf_len - delay_i) % buf_len;
            let r1 = (self.write + buf_len - delay_i - 1) % buf_len;
            wet_sum += self.buffer[r0] * (1.0 - frac) + self.buffer[r1] * frac;
        }

        self.write = (self.write + 1) % buf_len;

        sample * (1.0 - self.wet) + (wet_sum / 2.0) * self.wet
    }
}

// ═══════════════════════════════════════════════════════════════
//  Schroeder Reverb
// ═══════════════════════════════════════════════════════════════

/// Lightweight Schroeder reverb (4 comb + 2 all-pass).
#[frb(opaque)]
#[derive(Clone)]
pub struct ReverbEffect {
    /// Room size (0.0 = small, 1.0 = large).
    pub room_size: f32,
    /// High-frequency damping (0.0 = none, 1.0 = full).
    pub damping: f32,
    /// Dry/wet mix.
    pub wet: f32,
    pub bypass: bool,

    combs: [CombFilter; 4],
    allpasses: [AllPassFilter; 2],
}

impl ReverbEffect {
    /// Typical settings: room_size=0.5, damping=0.5, wet=0.3
    pub fn new(room_size: f32, damping: f32, wet: f32, sample_rate: f32) -> Self {
        // Delay lengths tuned for 44100 Hz, scaled for other rates
        let scale = sample_rate / 44100.0;
        let comb_delays = [1557, 1617, 1491, 1422];
        let ap_delays   = [225, 341];

        let feedback = 0.84 * room_size.clamp(0.0, 1.0) + 0.1;

        Self {
            room_size,
            damping: damping.clamp(0.0, 1.0),
            wet: wet.clamp(0.0, 1.0),
            bypass: false,
            combs: comb_delays.map(|d| CombFilter::new((d as f32 * scale) as usize, feedback, damping)),
            allpasses: ap_delays.map(|d| AllPassFilter::new((d as f32 * scale) as usize, 0.5)),
        }
    }

    pub fn update_params(&mut self) {
        let feedback = 0.84 * self.room_size.clamp(0.0, 1.0) + 0.1;
        for c in &mut self.combs {
            c.feedback = feedback;
            c.damp = self.damping;
        }
    }

    pub fn reset(&mut self) {
        for c in &mut self.combs    { c.buf.fill(0.0); c.pos = 0; c.last = 0.0; }
        for a in &mut self.allpasses { a.buf.fill(0.0); a.pos = 0; }
    }

    #[inline]
    pub fn process(&mut self, sample: f32) -> f32 {
        if self.bypass { return sample; }

        // Parallel comb filters
        let mut out = 0.0_f32;
        for c in &mut self.combs {
            out += c.process(sample);
        }

        // Series all-pass filters
        for a in &mut self.allpasses {
            out = a.process(out);
        }

        sample * (1.0 - self.wet) + out * self.wet * 0.25
    }
}

// ─────────────────────────────────────────────────────────────
//  Schroeder building blocks
// ─────────────────────────────────────────────────────────────
#[frb(opaque)]
#[derive(Clone)]
pub struct CombFilter {
    buf: Vec<f32>,
    pos: usize,
    feedback: f32,
    damp: f32,
    last: f32,
}

impl CombFilter {
    fn new(len: usize, feedback: f32, damp: f32) -> Self {
        Self { buf: vec![0.0; len.max(1)], pos: 0, feedback, damp, last: 0.0 }
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let output = self.buf[self.pos];
        self.last = output * (1.0 - self.damp) + self.last * self.damp;
        self.buf[self.pos] = input + self.last * self.feedback;
        self.pos = (self.pos + 1) % self.buf.len();
        output
    }
}

#[frb(opaque)]
#[derive(Clone)]
struct AllPassFilter {
    buf: Vec<f32>,
    pos: usize,
    feedback: f32,
}

impl AllPassFilter {
    fn new(len: usize, feedback: f32) -> Self {
        Self { buf: vec![0.0; len.max(1)], pos: 0, feedback }
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let bufout = self.buf[self.pos];
        let output = -input + bufout;
        self.buf[self.pos] = input + bufout * self.feedback;
        self.pos = (self.pos + 1) % self.buf.len();
        output
    }
}

// ═══════════════════════════════════════════════════════════════
//  Effects chain
// ═══════════════════════════════════════════════════════════════

/// Convenience wrapper that runs multiple effects in series.
///
/// Build an `EffectsChain` with whichever effects you need,
/// then call `process` once per sample in your audio callback.
#[frb(opaque)]
#[derive(Clone)]
pub struct EffectsChain {
    pub gain:       Option<GainEffect>,
    pub overdrive:  Option<OverdriveEffect>,
    pub hard_clip:  Option<HardClipEffect>,
    pub compressor: Option<CompressorEffect>,
    pub delay:      Option<DelayEffect>,
    pub chorus:     Option<ChorusEffect>,
    pub reverb:     Option<ReverbEffect>,
}

impl EffectsChain {
    /// Create an empty chain (all slots disabled).
    pub fn empty() -> Self {
        Self {
            gain:       None,
            overdrive:  None,
            hard_clip:  None,
            compressor: None,
            delay:      None,
            chorus:     None,
            reverb:     None,
        }
    }

    /// Process a single sample through every enabled effect in order:
    ///
    /// gain → overdrive → hard_clip → compressor → delay → chorus → reverb
    #[inline]
    pub fn process(&mut self, mut sample: f32) -> f32 {
        if let Some(e) = &mut self.gain       { sample = e.process(sample); }
        if let Some(e) = &mut self.overdrive  { sample = e.process(sample); }
        if let Some(e) = &mut self.hard_clip  { sample = e.process(sample); }
        if let Some(e) = &mut self.compressor { sample = e.process(sample); }
        if let Some(e) = &mut self.delay      { sample = e.process(sample); }
        if let Some(e) = &mut self.chorus     { sample = e.process(sample); }
        if let Some(e) = &mut self.reverb     { sample = e.process(sample); }
        sample
    }
}

// ─────────────────────────────────────────────────────────────
//  Math helpers
// ─────────────────────────────────────────────────────────────

#[inline]
pub fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

#[inline]
pub fn linear_to_db(lin: f32) -> f32 {
    20.0 * lin.abs().max(1e-9).log10()
}

/// Convert a time constant to a one-pole IIR filter coefficient.
#[inline]
fn time_to_coeff(time_s: f32, sample_rate: f32) -> f32 {
    if time_s <= 0.0 { 0.0 } else { (-1.0 / (time_s * sample_rate)).exp() }
}