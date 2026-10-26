use flutter_rust_bridge::frb;

/// Biquad filter coefficients (Direct Form I).
///
/// Transfer function:
///   H(z) = (b0 + b1*z⁻¹ + b2*z⁻²) / (1 + a1*z⁻¹ + a2*z⁻²)
#[derive(Debug, Clone, Copy)]
#[frb(opaque)]
pub struct BiquadCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

/// Biquad filter state (two delay elements per Direct Form I).
#[derive(Debug, Clone, Copy, Default)]
#[frb(opaque)]
pub struct BiquadState {
    x1: f32, // x[n-1]
    x2: f32, // x[n-2]
    y1: f32, // y[n-1]
    y2: f32, // y[n-2]
}

/// A single biquad filter stage with hot-swappable coefficients.
#[derive(Debug, Clone)]
#[frb(opaque)]
pub struct BiquadFilter {
    pub coeffs: BiquadCoeffs,
    state: BiquadState,
}

impl BiquadFilter {
    pub fn new(coeffs: BiquadCoeffs) -> Self {
        Self {
            coeffs,
            state: BiquadState::default(),
        }
    }

    /// Process one sample through the filter. Direct Form I.
    #[inline(always)]
    pub fn process(&mut self, x: f32) -> f32 {
        let BiquadCoeffs { b0, b1, b2, a1, a2 } = self.coeffs;
        let BiquadState { x1, x2, y1, y2 } = self.state;

        let y = b0 * x + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;

        self.state = BiquadState {
            x1: x,
            x2: x1,
            y1: y,
            y2: y1,
        };
        y
    }

    /// Replace coefficients without resetting delay state (glitch-free update).
    pub fn update_coeffs(&mut self, coeffs: BiquadCoeffs) {
        self.coeffs = coeffs;
    }

    pub fn reset(&mut self) {
        self.state = BiquadState::default();
    }
}

#[inline]
fn sanitise(fc: f32, q: f32, fs: f32) -> (f32, f32) {
    let fs = if fs.is_finite() && fs > 0.0 {
        fs
    } else {
        44_100.0
    };
    let fc = fc.clamp(1.0, fs * 0.49);
    let q = if q.is_finite() {
        q.clamp(0.05, 20.0)
    } else {
        0.707
    };
    (fc, q)
}

// ─────────────────────────────────────────────────────────────
//  Coefficient factories
// ─────────────────────────────────────────────────────────────

/// Peaking EQ band (boost/cut at a centre frequency).
///
/// * `fc`      – centre frequency in Hz
/// * `gain_db` – gain in dB (positive = boost, negative = cut)
/// * `q`       – quality factor (bandwidth). Typical value: 0.707–2.0
/// * `fs`      – sample rate in Hz
pub fn peaking_eq(fc: f32, gain_db: f32, q: f32, fs: f32) -> BiquadCoeffs {
    let (fc, q) = sanitise(fc, q, fs);
    let a = 10.0_f32.powf(gain_db / 40.0); // sqrt(10^(dB/20))
    let w0 = 2.0 * std::f32::consts::PI * fc / fs;
    let (sin_w0, cos_w0) = w0.sin_cos();
    let alpha = sin_w0 / (2.0 * q);

    let b0 = 1.0 + alpha * a;
    let b1 = -2.0 * cos_w0;
    let b2 = 1.0 - alpha * a;
    let a0 = 1.0 + alpha / a;
    let a1 = -2.0 * cos_w0;
    let a2 = 1.0 - alpha / a;

    normalise(b0, b1, b2, a0, a1, a2)
}

/// Low-shelf filter.
pub fn low_shelf(fc: f32, gain_db: f32, q: f32, fs: f32) -> BiquadCoeffs {
    let a = 10.0_f32.powf(gain_db / 40.0);
    let w0 = 2.0 * std::f32::consts::PI * fc / fs;
    let (sin_w0, cos_w0) = w0.sin_cos();
    let alpha = sin_w0 / (2.0 * q);
    let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

    let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha);
    let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
    let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha);
    let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
    let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
    let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha;

    normalise(b0, b1, b2, a0, a1, a2)
}

/// High-shelf filter.
pub fn high_shelf(fc: f32, gain_db: f32, q: f32, fs: f32) -> BiquadCoeffs {
    let a = 10.0_f32.powf(gain_db / 40.0);
    let w0 = 2.0 * std::f32::consts::PI * fc / fs;
    let (sin_w0, cos_w0) = w0.sin_cos();
    let alpha = sin_w0 / (2.0 * q);
    let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

    let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha);
    let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
    let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha);
    let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
    let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
    let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha;

    normalise(b0, b1, b2, a0, a1, a2)
}

/// Second-order high-pass filter.
pub fn high_pass(fc: f32, q: f32, fs: f32) -> BiquadCoeffs {
    let w0 = 2.0 * std::f32::consts::PI * fc / fs;
    let (sin_w0, cos_w0) = w0.sin_cos();
    let alpha = sin_w0 / (2.0 * q);

    let b0 = (1.0 + cos_w0) / 2.0;
    let b1 = -(1.0 + cos_w0);
    let b2 = (1.0 + cos_w0) / 2.0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_w0;
    let a2 = 1.0 - alpha;

    normalise(b0, b1, b2, a0, a1, a2)
}

/// Second-order low-pass filter.
pub fn low_pass(fc: f32, q: f32, fs: f32) -> BiquadCoeffs {
    let w0 = 2.0 * std::f32::consts::PI * fc / fs;
    let (sin_w0, cos_w0) = w0.sin_cos();
    let alpha = sin_w0 / (2.0 * q);

    let b0 = (1.0 - cos_w0) / 2.0;
    let b1 = 1.0 - cos_w0;
    let b2 = (1.0 - cos_w0) / 2.0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_w0;
    let a2 = 1.0 - alpha;

    normalise(b0, b1, b2, a0, a1, a2)
}

/// Normalise all coefficients by a0.
#[inline]
fn normalise(b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) -> BiquadCoeffs {
    BiquadCoeffs {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
    }
}
