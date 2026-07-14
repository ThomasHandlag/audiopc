/// Per-sample DSP processing trait.
///
/// Every audio effect that can be inserted into the signal chain implements
/// this trait.  All methods are called on the **audio callback thread** so
/// implementations MUST be:
///
/// * **Real-time safe** — no heap allocation, no blocking I/O, no mutex.
/// * `Send` — the engine may move processors between threads during
///   construction / reconfiguration.
///
/// # Scalability
///
/// To add a new effect:
/// 1. Implement `AudioProcessor` for your type.
/// 2. Wrap it in `Box<dyn AudioProcessor>` and push it into the
///    `Effects::chain` vector using [`Effects::push`].
///
/// No changes to the engine or FFI layer are needed.
pub trait AudioProcessor: Send + 'static {
    /// Process a single sample in-place.
    ///
    /// The `channel` argument is the channel index (0-based) of the sample
    /// in the current interleaved frame.  Most effects ignore it; stereo-
    /// aware effects (panning, mid-side) can use it for per-channel state.
    fn process_sample(&mut self, sample: f32, channel: usize) -> f32;

    /// Called when the stream format changes (sample rate or channel count).
    ///
    /// Implementations should re-compute filter coefficients here.
    /// The default no-ops are correct for stateless or rate-agnostic effects.
    #[allow(unused_variables)]
    fn reset(&mut self, sample_rate: u32, channels: u16) {}

    /// Human-readable name, useful for debugging and serialisation.
    fn name(&self) -> &'static str;
}

// ── Built-in biquad processors ────────────────────────────────────────────────

use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type as BiquadType};

/// A single biquad filter with one coefficient set.
///
/// Used as the building block for `LowPass`, `HighPass`, `Peak`, etc.
pub struct BiquadFilter {
    filter: Option<DirectForm1<f32>>,
    coeffs: Coefficients<f32>,
    label:  &'static str,
}

impl BiquadFilter {
    pub fn new(coeffs: Coefficients<f32>, label: &'static str) -> Self {
        Self {
            filter: Some(DirectForm1::new(coeffs)),
            coeffs,
            label,
        }
    }
}

impl AudioProcessor for BiquadFilter {
    #[inline]
    fn process_sample(&mut self, sample: f32, _channel: usize) -> f32 {
        if let Some(f) = &mut self.filter {
            f.run(sample)
        } else {
            sample
        }
    }

    fn reset(&mut self, _sample_rate: u32, _channels: u16) {
        // Re-initialise state; keep same coefficients.
        self.filter = Some(DirectForm1::new(self.coeffs));
    }

    fn name(&self) -> &'static str { self.label }
}

/// A linear gain / volume node.
pub struct GainNode {
    /// Linear gain factor (1.0 = unity, 2.0 = +6 dB, 0.5 = −6 dB).
    pub gain: f32,
}

impl GainNode {
    pub fn new(gain: f32) -> Self { Self { gain } }
}

impl AudioProcessor for GainNode {
    #[inline]
    fn process_sample(&mut self, sample: f32, _channel: usize) -> f32 {
        sample * self.gain
    }
    fn name(&self) -> &'static str { "GainNode" }
}

// ── Effects chain ─────────────────────────────────────────────────────────────

/// An ordered chain of [`AudioProcessor`] instances applied to a single
/// output channel.
///
/// The chain applies processors in **insertion order**.  Use [`Effects::push`]
/// to add a new processor, [`Effects::clear`] to remove all, and
/// [`Effects::remove_named`] to remove by name.
///
/// # Example
/// ```ignore
/// // Low-pass at 8 kHz, Q = 0.71
/// let coeffs = Coefficients::<f32>::from_params(
///     biquad::Type::LowPass, 44100.hz(), 8000.hz(), 0.71,
/// ).unwrap();
/// channel_effects.push(BiquadFilter::new(coeffs, "LowPass"));
/// ```
pub struct Effects {
    pub chain: Vec<Box<dyn AudioProcessor>>,
}

impl Effects {
    /// Create an empty effect chain (no processing → unity gain).
    pub fn new() -> Self {
        Self { chain: Vec::new() }
    }

    /// Append a new processor to the end of the chain.
    pub fn push<P: AudioProcessor>(&mut self, processor: P) {
        self.chain.push(Box::new(processor));
    }

    /// Remove all processors, restoring bypass mode.
    pub fn clear(&mut self) {
        self.chain.clear();
    }

    /// Remove the first processor with the given name.
    /// Returns `true` if one was removed.
    pub fn remove_named(&mut self, name: &str) -> bool {
        if let Some(pos) = self.chain.iter().position(|p| p.name() == name) {
            self.chain.remove(pos);
            true
        } else {
            false
        }
    }

    /// Whether the chain is empty (pure bypass).
    pub fn is_empty(&self) -> bool { self.chain.is_empty() }

    /// Process a single sample through the full chain.
    ///
    /// Inlined for the audio-callback hot path.
    #[inline]
    pub fn process(&mut self, sample: f32, channel: usize) -> f32 {
        let mut s = sample;
        for processor in &mut self.chain {
            s = processor.process_sample(s, channel);
        }
        s
    }

    /// Propagate a format change to every processor in the chain.
    pub fn reset_all(&mut self, sample_rate: u32, channels: u16) {
        for p in &mut self.chain {
            p.reset(sample_rate, channels);
        }
    }
}

impl Default for Effects { fn default() -> Self { Self::new() } }

// ── Convenience constructors for the most common biquad types ─────────────────

/// Helper: build a biquad filter and box it, or return `None` on error.
fn make_biquad(
    kind:        BiquadType<f32>,
    sample_rate: u32,
    cutoff_hz:   f32,
    q:           f32,
    label:       &'static str,
) -> Option<BiquadFilter> {
    Coefficients::<f32>::from_params(kind, sample_rate.hz(), cutoff_hz.hz(), q)
        .ok()
        .map(|c| BiquadFilter::new(c, label))
}

/// Build a peaking-EQ filter (`+gain dB` at `cutoff_hz`).
pub fn peak_filter(sample_rate: u32, cutoff_hz: f32, gain_db: f32, q: f32)
    -> Option<BiquadFilter>
{
    make_biquad(BiquadType::PeakingEQ(gain_db), sample_rate, cutoff_hz, q, "PeakEQ")
}

/// Build a low-shelf filter.
pub fn low_shelf_filter(sample_rate: u32, cutoff_hz: f32, gain_db: f32, q: f32)
    -> Option<BiquadFilter>
{
    make_biquad(BiquadType::LowShelf(gain_db), sample_rate, cutoff_hz, q, "LowShelf")
}

/// Build a high-shelf filter.
pub fn high_shelf_filter(sample_rate: u32, cutoff_hz: f32, gain_db: f32, q: f32)
    -> Option<BiquadFilter>
{
    make_biquad(BiquadType::HighShelf(gain_db), sample_rate, cutoff_hz, q, "HighShelf")
}

/// Build a second-order band-pass filter.
pub fn band_pass_filter(sample_rate: u32, center_hz: f32, q: f32)
    -> Option<BiquadFilter>
{
    make_biquad(BiquadType::BandPass, sample_rate, center_hz, q, "BandPass")
}

/// Build a notch (band-reject) filter.
pub fn notch_filter(sample_rate: u32, cutoff_hz: f32, q: f32)
    -> Option<BiquadFilter>
{
    make_biquad(BiquadType::Notch, sample_rate, cutoff_hz, q, "Notch")
}

/// Build a second-order Butterworth low-pass filter.
pub fn lowpass_filter(sample_rate: u32, cutoff_hz: f32, q: f32)
    -> Option<BiquadFilter>
{
    make_biquad(BiquadType::LowPass, sample_rate, cutoff_hz, q, "LowPass")
}

/// Build a second-order Butterworth high-pass filter.
pub fn highpass_filter(sample_rate: u32, cutoff_hz: f32, q: f32)
    -> Option<BiquadFilter>
{
    make_biquad(BiquadType::HighPass, sample_rate, cutoff_hz, q, "HighPass")
}