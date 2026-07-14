/// All public constants used across the crate.
///
/// Centralised here so that feature-gating and per-platform tuning is easy.

// ── Queue sizing ──────────────────────────────────────────────────────────────

/// Default number of seconds worth of decoded samples to keep buffered.
pub const DEFAULT_MAX_QUEUE_SECONDS: usize = 20;
/// Minimum allowed value for `max_queue_seconds`.
pub const MIN_MAX_QUEUE_SECONDS: usize = 1;
/// Maximum allowed value for `max_queue_seconds`.
pub const MAX_MAX_QUEUE_SECONDS: usize = 120;

// ── Decode thread back-pressure ───────────────────────────────────────────────

/// How long (ms) the decode thread sleeps when the queue is full, before
/// retrying.  Keeping this short keeps latency low while still yielding to
/// the scheduler.
pub const DECODE_BACKPRESSURE_SLEEP_MS: u64 = 2;

// ── Visualizer ────────────────────────────────────────────────────────────────

/// Number of seconds of audio kept in the visualizer ring buffer.
pub const DEFAULT_VISUALIZER_SECONDS: usize = 2;
/// FFT size used by `VisualizerProcessor`.  Must be a power of two.
pub const VISUALIZER_FFT_SIZE: usize = 2048;
/// Default number of output frequency bars for the spectrum visualizer.
pub const DEFAULT_VISUALIZER_BAR_COUNT: usize = 64;
/// Lowest frequency bucket shown on the visualizer (Hz).
pub const VISUALIZER_MIN_HZ: f32 = 35.0;

// ── Playback rate ─────────────────────────────────────────────────────────────

/// Minimum allowed playback rate (0.5 = half speed).
pub const MIN_RATE: f32 = 0.5;
/// Maximum allowed playback rate (2.0 = double speed).
pub const MAX_RATE: f32 = 2.0;

// ── Device watcher ────────────────────────────────────────────────────────────

/// How frequently (ms) the device watcher thread polls for device changes.
/// This value is a trade-off between responsiveness and CPU usage.
pub const DEVICE_POLL_INTERVAL_MS: u64 = 2_000;