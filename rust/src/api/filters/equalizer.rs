use flutter_rust_bridge::frb;

use crate::api::filters::biquad::{self, BiquadFilter};

/// The shape of an EQ band.
#[derive(Debug, Clone, Copy, PartialEq)]
#[frb(opaque)]
pub enum BandKind {
    /// Bell / peaking (boost or cut around a centre frequency)
    Peaking,
    /// Low-shelf (boost/cut everything below fc)
    LowShelf,
    /// High-shelf (boost/cut everything above fc)
    HighShelf,
    /// Second-order high-pass (roll off below fc)
    HighPass,
    /// Second-order low-pass (roll off above fc)
    LowPass,
}

/// All parameters describing a single EQ band.
#[derive(Debug, Clone, Copy)]
#[frb(opaque)]
pub struct BandConfig {
    pub kind: BandKind,
    /// Centre / corner frequency in Hz.
    pub freq: f32,
    /// Gain in dB. Ignored for `HighPass` / `LowPass`.
    pub gain_db: f32,
    /// Quality factor (0.1 – 10, typical 0.707).
    pub q: f32,
    /// If false, the band is bypassed (unity gain).
    pub enabled: bool,
}

impl BandConfig {
    /// A sensible peaking band centred at `freq`.
    pub fn peaking(freq: f32, gain_db: f32, q: f32) -> Self {
        Self { kind: BandKind::Peaking, freq, gain_db, q, enabled: true }
    }
    pub fn low_shelf(freq: f32, gain_db: f32, q: f32) -> Self {
        Self { kind: BandKind::LowShelf, freq, gain_db, q, enabled: true }
    }
    pub fn high_shelf(freq: f32, gain_db: f32, q: f32) -> Self {
        Self { kind: BandKind::HighShelf, freq, gain_db, q, enabled: true }
    }
    pub fn high_pass(freq: f32, q: f32) -> Self {
        Self { kind: BandKind::HighPass, freq, gain_db: 0.0, q, enabled: true }
    }
    pub fn low_pass(freq: f32, q: f32) -> Self {
        Self { kind: BandKind::LowPass, freq, gain_db: 0.0, q, enabled: true }
    }
}

/// One EQ band: its config + the live biquad filter.
#[frb(opaque)]
#[derive(Clone)]
struct Band {
    config: BandConfig,
    filter: BiquadFilter,
}

impl Band {
    fn new(config: BandConfig, sample_rate: f32) -> Self {
        let filter = BiquadFilter::new(coeffs_for(&config, sample_rate));
        Self { config, filter }
    }

    fn update(&mut self, config: BandConfig, sample_rate: f32) {
        self.config = config;
        self.filter.update_coeffs(coeffs_for(&config, sample_rate));
    }

    #[inline(always)]
    fn process(&mut self, x: f32) -> f32 {
        if self.config.enabled {
            self.filter.process(x)
        } else {
            x
        }
    }
}

/// An N-band parametric equalizer.
///
/// # flutter_rust_bridge usage
/// ```dart
/// final eq = await Equalizer.create(sampleRate: 44100);
/// await eq.addBand(BandConfig(...));
/// final out = eq.processSample(sample);
/// ```
#[frb(opaque)]
#[derive(Clone)]
pub struct Equalizer {
    bands: Vec<Band>,
    sample_rate: f32,
    /// Master bypass.
    pub bypass: bool,
}

impl Equalizer {
    /// Create an empty equalizer.
    #[frb(sync)]
    pub fn new(sample_rate: f32) -> Self {
        Self { bands: Vec::new(), sample_rate, bypass: false }
    }

    /// Convenience: create a ready-to-use 10-band graphic EQ.
    ///
    /// Bands at: 31, 63, 125, 250, 500, 1k, 2k, 4k, 8k, 16k Hz — all at 0 dB.
    #[frb(sync)]
    pub fn new_graphic_10band(sample_rate: f32) -> Self {
        let freqs = [31.0, 63.0, 125.0, 250.0, 500.0, 1_000.0,
                     2_000.0, 4_000.0, 8_000.0, 16_000.0_f32];
        let mut eq = Self::new(sample_rate);
        for &f in &freqs {
            eq.add_band(BandConfig::peaking(f, 0.0, 1.41));
        }
        eq
    }

    /// Add a new band at the end of the chain.
    #[frb(sync)]
    pub fn add_band(&mut self, config: BandConfig) {
        self.bands.push(Band::new(config, self.sample_rate));
    }

    /// Replace configuration for band `index` (zero-based).
    /// Returns `false` if the index is out of range.
    #[frb(sync)]
    pub fn set_band(&mut self, index: usize, config: BandConfig) -> bool {
        if let Some(band) = self.bands.get_mut(index) {
            band.update(config, self.sample_rate);
            true
        } else {
            false
        }
    }

    /// Convenience: just change the gain of an existing band.
    #[frb(sync)]
    pub fn set_band_gain(&mut self, index: usize, gain_db: f32) -> bool {
        if let Some(band) = self.bands.get_mut(index) {
            let mut cfg = band.config;
            cfg.gain_db = gain_db;
            band.update(cfg, self.sample_rate);
            true
        } else {
            false
        }
    }

    /// Enable / disable a single band without rebuilding coefficients.
    #[frb(sync)]
    pub fn set_band_enabled(&mut self, index: usize, enabled: bool) -> bool {
        if let Some(band) = self.bands.get_mut(index) {
            band.config.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Get a copy of a band's current configuration.
    #[frb(sync)]
    pub fn band_config(&self, index: usize) -> Option<BandConfig> {
        self.bands.get(index).map(|b| b.config)
    }

    #[frb(sync)]
    pub fn band_count(&self) -> usize {
        self.bands.len()
    }

    #[frb(sync)]
    pub fn remove_band(&mut self, index: usize) {
        if index < self.bands.len() {
            self.bands.remove(index);
        }
    }

    pub fn clear_bands(&mut self) {
        self.bands.clear();
    }

    /// Reset all filter delay states (call after a seek or stream switch).
    pub fn reset(&mut self) {
        for b in &mut self.bands {
            b.filter.reset();
        }
    }

    /// Update sample rate and recompute all coefficients.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        let cfgs: Vec<BandConfig> = self.bands.iter().map(|b| b.config).collect();
        for (band, cfg) in self.bands.iter_mut().zip(cfgs) {
            band.update(cfg, sample_rate);
        }
    }

    /// **Core DSP entry point** – process a single `f32` sample.
    ///
    /// All bands are applied in series. With no bands, or when bypassed,
    /// returns `sample` unchanged.
    #[inline]
    pub fn process(&mut self, sample: f32) -> f32 {
        if self.bypass {
            return sample;
        }
        let mut s = sample;
        for band in &mut self.bands {
            s = band.process(s);
        }
        s
    }
}

// ─────────────────────────────────────────────────────────────
//  Helper
// ─────────────────────────────────────────────────────────────

fn coeffs_for(cfg: &BandConfig, fs: f32) -> crate::api::filters::biquad::BiquadCoeffs {
    match cfg.kind {
        BandKind::Peaking   => biquad::peaking_eq(cfg.freq, cfg.gain_db, cfg.q, fs),
        BandKind::LowShelf  => biquad::low_shelf(cfg.freq, cfg.gain_db, cfg.q, fs),
        BandKind::HighShelf => biquad::high_shelf(cfg.freq, cfg.gain_db, cfg.q, fs),
        BandKind::HighPass  => biquad::high_pass(cfg.freq, cfg.q, fs),
        BandKind::LowPass   => biquad::low_pass(cfg.freq, cfg.q, fs),
    }
}