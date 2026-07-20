use flutter_rust_bridge::frb;
use rustfft::{Fft, FftPlanner, num_complex::Complex};

/// Enum representing different frequency bands / instruments.
/// Each variant knows its own frequency range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BandKind {
    // Standard / Detailed Bands
    SubBass,
    Bass,
    UpperBass,
    LowMid,
    Mid,
    UpperMid,
    Presence,
    High,
    Brilliance,
    Air,
    // Instrument Bands
    Kick,
    SnareBody,
    SnareCrack,
    Toms,
    Vocal,
    HiHat,
    Cymbals,
}

impl BandKind {
    /// Returns the (min_hz, max_hz) frequency range for this band kind.
    pub fn freq_range(&self) -> (f32, f32) {
        match self {
            BandKind::SubBass => (20.0, 60.0),
            BandKind::Bass => (60.0, 250.0),
            BandKind::UpperBass => (150.0, 300.0),
            BandKind::LowMid => (250.0, 500.0),
            BandKind::Mid => (500.0, 2000.0),
            BandKind::UpperMid => (2000.0, 4000.0),
            BandKind::Presence => (4000.0, 6000.0),
            BandKind::High => (4800.0, 8000.0),
            BandKind::Brilliance => (6000.0, 12000.0),
            BandKind::Air => (12000.0, 20000.0),
            BandKind::Kick => (40.0, 100.0),
            BandKind::SnareBody => (150.0, 300.0),
            BandKind::SnareCrack => (2000.0, 5000.0),
            BandKind::Toms => (80.0, 350.0),
            BandKind::Vocal => (300.0, 3400.0),
            BandKind::HiHat => (6000.0, 15000.0),
            BandKind::Cymbals => (5000.0, 18000.0),
        }
    }

    /// Standard frequency bands covering the full audible spectrum.
    pub fn standard() -> Vec<BandKind> {
        vec![
            BandKind::SubBass,
            BandKind::Bass,
            BandKind::LowMid,
            BandKind::Mid,
            BandKind::UpperMid,
            BandKind::Presence,
            BandKind::Brilliance,
            BandKind::Air,
        ]
    }

    /// Instrument-approximation bands (overlapping ranges).
    pub fn instrument() -> Vec<BandKind> {
        vec![
            BandKind::Kick,
            BandKind::Bass,
            BandKind::SnareBody,
            BandKind::SnareCrack,
            BandKind::Toms,
            BandKind::Vocal,
            BandKind::HiHat,
            BandKind::Cymbals,
        ]
    }

    /// Detailed 10-band breakdown with finer mid-range resolution.
    pub fn detailed() -> Vec<BandKind> {
        vec![
            BandKind::SubBass,
            BandKind::Bass,
            BandKind::UpperBass,
            BandKind::LowMid,
            BandKind::Mid,
            BandKind::UpperMid,
            BandKind::Presence,
            BandKind::High,
            BandKind::Brilliance,
            BandKind::Air,
        ]
    }
}

/// Computed energy metrics for a single frequency band.
#[derive(Debug, Clone)]
pub struct BandEnergy {
    /// The band category analyzed.
    pub kind: BandKind,
    /// Lower frequency bound in Hz.
    pub min_hz: f32,
    /// Upper frequency bound in Hz.
    pub max_hz: f32,
    /// Raw average magnitude in this band this frame.
    pub raw: f32,
    /// Time-smoothed energy (fast attack, slow release).
    pub smoothed: f32,
    /// Peak hold value (slowly decaying).
    pub peak: f32,
    /// Normalized energy (0–1), relative to this band's adaptive level.
    pub normalized: f32,
    /// Transient / onset strength (0–1). Higher = percussive hit detected.
    pub beat: f32,
}

/// Internal per-band smoothing state.
#[derive(Clone)]
pub struct BandState {
   pub smoothed: f32,
   pub peak: f32,
   pub fast_energy: f32,
   pub slow_energy: f32,
   pub adaptive_level: f32,
}

impl Default for BandState {
    fn default() -> Self {
        Self {
            smoothed: 0.0,
            peak: 0.0,
            fast_energy: 0.0,
            slow_energy: 0.0,
            adaptive_level: 0.001,
        }
    }
}

// ─── Visualizer Processor ────────────────────────────────────────────
#[frb(opaque)]
pub struct VisualizerProcessor {
    fft: std::sync::Arc<dyn Fft<f32>>,
    fft_buffer: Vec<Complex<f32>>,
    smoothed_bars: Vec<f32>,
    adaptive_level: f32,
    fast_energy: f32,
    slow_energy: f32,
    // Band analysis cache
    last_magnitudes: Vec<f32>,
    last_sample_rate: u32,
    band_states: Vec<BandState>,
    fft_size: usize,
    bar_count: usize
}

impl VisualizerProcessor {
    #[frb(sync)]
    pub fn new(bar_count: usize, fft_size: usize) -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(fft_size.max(512));
        Self {
            fft,
            fft_buffer: vec![Complex::ZERO; fft_size],
            smoothed_bars: vec![0.0; bar_count.max(24)],
            adaptive_level: 0.08,
            fast_energy: 0.0,
            slow_energy: 0.0,
            last_magnitudes: Vec::new(),
            last_sample_rate: 0,
            band_states: Vec::new(),
            fft_size,
            bar_count
        }
    }

    pub fn reset(&mut self) {
        self.smoothed_bars.fill(0.0);
        self.adaptive_level = 0.08;
        self.fast_energy = 0.0;
        self.slow_energy = 0.0;
        self.last_magnitudes.clear();
        self.last_sample_rate = 0;
        for state in &mut self.band_states {
            *state = BandState::default();
        }
    }

    pub fn ensure_bar_count(&mut self, count: usize) {
        let count = count.max(1);
        if self.smoothed_bars.len() != count {
            self.smoothed_bars = vec![0.0; count];
            self.adaptive_level = 0.08;
        }
    }

    fn ensure_band_states(&mut self, count: usize) {
        if self.band_states.len() < count {
            self.band_states.resize(count, BandState::default());
        }
    }

    pub fn decay_only(&mut self) -> Vec<f32> {
        self.ensure_bar_count(self.bar_count);

        for value in &mut self.smoothed_bars {
            *value *= 0.93;
        }

        self.fast_energy *= 0.90;
        self.slow_energy *= 0.97;

        for state in &mut self.band_states {
            state.smoothed *= 0.93;
            state.peak *= 0.97;
            state.fast_energy *= 0.90;
            state.slow_energy *= 0.97;
            state.adaptive_level = (state.adaptive_level * 0.98).max(0.0001);
        }

        self.smoothed_bars.clone()
    }

    fn perform_fft(
        &mut self,
        samples: &[f32],
        channels: usize,
        sample_rate: u32,
    ) -> (Vec<f32>, f32) {
        let frame_count = samples.len() / channels;
        let window_frames = self.fft_size.min(frame_count);
        let start_frame = frame_count.saturating_sub(window_frames);

        self.fft_buffer.fill(Complex::ZERO);

        let mut rms_acc = 0.0f32;
        for index in 0..window_frames {
            let src_frame = start_frame + index;
            let mut mixed = 0.0f32;
            for ch in 0..channels {
                mixed += samples[src_frame * channels + ch];
            }
            let mono = mixed / channels as f32;
            rms_acc += mono * mono;

            let target = self.fft_size - window_frames + index;
            self.fft_buffer[target].re = mono * hann_window(index, window_frames);
        }

        self.fft.process(&mut self.fft_buffer);

        let half = self.fft_size / 2;
        let norm = window_frames.max(1) as f32;
        let magnitudes: Vec<f32> = self
            .fft_buffer
            .iter()
            .take(half)
            .map(|v| (v.re * v.re + v.im * v.im).sqrt() / norm)
            .collect();

        let rms = (rms_acc / window_frames.max(1) as f32).sqrt();

        self.last_magnitudes = magnitudes.clone();
        self.last_sample_rate = sample_rate;

        (magnitudes, rms)
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Bar Visualization
    // ═══════════════════════════════════════════════════════════════════

    pub fn compute(
        &mut self,
        samples: &[f32],
        channels: usize,
        sample_rate: u32,
        playing: bool,
    ) -> Vec<f32> {
        if !playing || samples.is_empty() || channels == 0 || sample_rate == 0 {
            return self.decay_only();
        }

        let frame_count = samples.len() / channels;
        if frame_count == 0 {
            return self.decay_only();
        }

        let (magnitudes, rms) = self.perform_fft(samples, channels, sample_rate);

        self.fast_energy = self.fast_energy * 0.50 + rms * 0.50;
        self.slow_energy = self.slow_energy * 0.97 + rms * 0.03;
        let beat = ((self.fast_energy - self.slow_energy) * 11.0).clamp(0.0, 1.0);

        let min_hz = 20.0;
        let max_hz = ((sample_rate as f32) * 0.46).max(min_hz + 1.0);
        let bar_count = self.bar_count;
        let mut raw_bars = vec![0.0f32; bar_count];

        for bar in 0..bar_count {
            let t0 = bar as f32 / bar_count as f32;
            let t1 = (bar + 1) as f32 / bar_count as f32;
            let f0 = log_interp(min_hz, max_hz, t0);
            let f1 = log_interp(min_hz, max_hz, t1);

            let b0 = hz_to_bin(f0, sample_rate, self.fft_size).max(1);
            let b1 = hz_to_bin(f1, sample_rate, self.fft_size).max(b0 + 1);
            let end = b1.min(magnitudes.len());
            let start = b0.min(end.saturating_sub(1));

            let mut energy = 0.0f32;
            let mut count = 0usize;
            for value in &magnitudes[start..end] {
                energy += *value;
                count += 1;
            }

            let mut raw = if count > 0 {
                energy / count as f32
            } else {
                0.0
            };
            raw *= 1.0 + beat * 0.55 * (1.0 - t0);
            raw_bars[bar] = raw;
        }

        let frame_peak = raw_bars.iter().copied().fold(0.0f32, f32::max);
        self.adaptive_level = self.adaptive_level * 0.95 + frame_peak.max(0.0001) * 0.05;
        let level = self.adaptive_level.max(0.0001);

        let mut spatial = vec![0.0f32; bar_count];
        for index in 0..bar_count {
            let left = if index > 0 {
                raw_bars[index - 1]
            } else {
                raw_bars[index]
            };
            let center = raw_bars[index];
            let right = if index + 1 < bar_count {
                raw_bars[index + 1]
            } else {
                raw_bars[index]
            };
            spatial[index] = left * 0.20 + center * 0.60 + right * 0.20;
        }

        let mut out = vec![0.0; bar_count];

        for (index, raw) in spatial.iter().enumerate() {
            let mut target = (raw / level).clamp(0.0, 2.0);
            target = target.powf(0.78) * 0.70;
            target = target.clamp(0.0, 1.0);

            let current = self.smoothed_bars[index];
            let alpha = if target > current { 0.34 } else { 0.08 };
            self.smoothed_bars[index] = current + (target - current) * alpha;

            out[index] = self.smoothed_bars[index];
        }

        out
    }

    /// Compute band energies using cached magnitudes from the last
    /// `compute()` or `analyze_bands()` call. Near-zero cost.
    pub fn compute_bands(&mut self, kinds: &[BandKind]) -> Vec<BandEnergy> {
        self.ensure_band_states(kinds.len());

        if self.last_magnitudes.is_empty() || self.last_sample_rate == 0 {
            return self.decay_bands(kinds);
        }

        let sample_rate = self.last_sample_rate;
        let half = self.last_magnitudes.len();

        let mut results = Vec::with_capacity(kinds.len());

        for (i, &kind) in kinds.iter().enumerate() {
            let (min_hz, max_hz) = kind.freq_range();

            let b0 = hz_to_bin(min_hz, sample_rate, self.fft_size);
            let b1 = hz_to_bin(max_hz, sample_rate, self.fft_size);
            let start = b0.min(half);
            let end = (b1.max(start + 1)).min(half);

            let mut energy = 0.0f32;
            let mut count = 0usize;

            if start < end {
                for value in &self.last_magnitudes[start..end] {
                    energy += *value;
                    count += 1;
                }
            }

            let raw = if count > 0 {
                energy / count as f32
            } else {
                0.0
            };

            let state = &mut self.band_states[i];

            state.fast_energy = state.fast_energy * 0.50 + raw * 0.50;
            state.slow_energy = state.slow_energy * 0.97 + raw * 0.03;
            let beat = ((state.fast_energy - state.slow_energy) * 11.0).clamp(0.0, 1.0);

            state.adaptive_level = state.adaptive_level * 0.95 + raw.max(0.0001) * 0.05;
            let level = state.adaptive_level.max(0.0001);

            let alpha = if raw > state.smoothed { 0.40 } else { 0.10 };
            state.smoothed = state.smoothed + (raw - state.smoothed) * alpha;

            if state.smoothed > state.peak {
                state.peak = state.smoothed;
            } else {
                state.peak *= 0.97;
            }

            let normalized = (state.smoothed / level).clamp(0.0, 1.0).powf(0.75);

            results.push(BandEnergy {
                kind,
                min_hz,
                max_hz,
                raw,
                smoothed: state.smoothed,
                peak: state.peak,
                normalized,
                beat,
            });
        }

        results
    }

    /// Standalone band analysis — performs its own FFT.
    pub fn analyze_bands(
        &mut self,
        samples: &[f32],
        channels: usize,
        sample_rate: u32,
        kinds: &[BandKind],
        playing: bool,
    ) -> Vec<BandEnergy> {
        self.ensure_band_states(kinds.len());

        if !playing || samples.is_empty() || channels == 0 || sample_rate == 0 {
            return self.decay_bands(kinds);
        }

        let frame_count = samples.len() / channels;
        if frame_count == 0 {
            return self.decay_bands(kinds);
        }

        let _ = self.perform_fft(samples, channels, sample_rate);

        self.compute_bands(kinds)
    }

    fn decay_bands(&mut self, kinds: &[BandKind]) -> Vec<BandEnergy> {
        self.ensure_band_states(kinds.len());

        let mut results = Vec::with_capacity(kinds.len());

        for (i, &kind) in kinds.iter().enumerate() {
            let (min_hz, max_hz) = kind.freq_range();
            let state = &mut self.band_states[i];

            state.smoothed *= 0.93;
            state.peak *= 0.97;
            state.fast_energy *= 0.90;
            state.slow_energy *= 0.97;
            state.adaptive_level = (state.adaptive_level * 0.98).max(0.0001);

            results.push(BandEnergy {
                kind,
                min_hz,
                max_hz,
                raw: 0.0,
                smoothed: state.smoothed,
                peak: state.peak,
                normalized: 0.0,
                beat: 0.0,
            });
        }

        results
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Convenience Aggregators
    // ═══════════════════════════════════════════════════════════════════

    /// Maximum beat value across bands matching the requested kinds.
    pub fn beat_for_kinds(bands: &[BandEnergy], kinds: &[BandKind]) -> f32 {
        bands
            .iter()
            .filter(|b| kinds.contains(&b.kind))
            .map(|b| b.beat)
            .fold(0.0f32, f32::max)
    }

    /// Composite drum hit strength (kick + snare body + snare crack + toms).
    pub fn drum_beat(bands: &[BandEnergy]) -> f32 {
        Self::beat_for_kinds(
            bands,
            &[
                BandKind::Kick,
                BandKind::SnareBody,
                BandKind::SnareCrack,
                BandKind::Toms,
            ],
        )
    }

    /// Composite bass hit strength (kick + bass).
    pub fn bass_beat(bands: &[BandEnergy]) -> f32 {
        Self::beat_for_kinds(bands, &[BandKind::Kick, BandKind::Bass])
    }

    /// Vocal presence as normalized energy.
    pub fn vocal_presence(bands: &[BandEnergy]) -> f32 {
        bands
            .iter()
            .find(|b| b.kind == BandKind::Vocal)
            .map(|b| b.normalized)
            .unwrap_or(0.0)
    }

    /// High-frequency energy (hi-hat, cymbals, sibilance) as max normalized
    /// value across bands starting at 4 kHz or above.
    pub fn high_frequency_energy(bands: &[BandEnergy]) -> f32 {
        bands
            .iter()
            .filter(|b| b.min_hz >= 4000.0)
            .map(|b| b.normalized)
            .fold(0.0f32, f32::max)
    }

    /// Low-frequency energy (sub bass + bass) as max normalized value
    /// across bands below 250 Hz.
    pub fn low_frequency_energy(bands: &[BandEnergy]) -> f32 {
        bands
            .iter()
            .filter(|b| b.max_hz <= 250.0)
            .map(|b| b.normalized)
            .fold(0.0f32, f32::max)
    }
}

// ─── Helper Functions ────────────────────────────────────────────────

fn hann_window(index: usize, len: usize) -> f32 {
    if len <= 1 {
        return 1.0;
    }
    let x = index as f32 / (len - 1) as f32;
    (0.5 - 0.5 * (2.0 * std::f32::consts::PI * x).cos()).clamp(0.0, 1.0)
}

fn log_interp(min: f32, max: f32, t: f32) -> f32 {
    min * (max / min).powf(t.clamp(0.0, 1.0))
}

fn hz_to_bin(freq: f32, sample_rate: u32, fft_size: usize) -> usize {
    let nyquist = sample_rate as f32 / 2.0;
    let clamped = freq.clamp(0.0, nyquist);
    ((clamped / sample_rate as f32) * fft_size as f32) as usize
}
