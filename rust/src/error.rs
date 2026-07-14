/// Unified, typed error type for the entire audio library.
///
/// Every public API that can fail returns `Result<_, AudioError>`.  
/// Internal helpers also use this type to keep the error chain coherent.  
///
/// Because the crate currently targets a C FFI surface the error is kept
/// as a plain enum (not trait-object based) so variants stay zero-cost and
/// the whole thing is `Send + 'static`.
#[derive(Debug, Clone, PartialEq)]
pub enum AudioError {
    /// No output device is present on this host.
    NoDevice,

    /// A named device could not be found.
    DeviceNotFound { name: String },

    /// The audio format or codec is not supported.
    UnsupportedFormat(String),

    /// A decode/codec error occurred.
    DecodeError(String),

    /// An I/O error occurred (file open, read, write …).
    Io(String),

    /// A network/HTTP error occurred.
    Network(String),

    /// Querying the stream configurations from the device failed.
    StreamConfig(String),

    /// Building the cpal output/input stream failed.
    StreamBuild(String),

    /// The source does not support seeking.
    SeekNotSupported,

    /// A seek target was outside the duration of the source.
    SeekOutOfRange { pos_ms: i32 },

    /// An error in the DSP/processing pipeline.
    Pipeline(String),

    /// No source has been loaded yet.
    NoSource,

    /// The engine's internal mutex was poisoned.
    Poisoned,
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDevice => write!(f, "No output device available"),
            Self::DeviceNotFound { name } => write!(f, "Device not found: {name}"),
            Self::UnsupportedFormat(s) => write!(f, "Unsupported format: {s}"),
            Self::DecodeError(s) => write!(f, "Decode error: {s}"),
            Self::Io(s) => write!(f, "I/O error: {s}"),
            Self::Network(s) => write!(f, "Network error: {s}"),
            Self::StreamConfig(s) => write!(f, "Stream config error: {s}"),
            Self::StreamBuild(s) => write!(f, "Stream build error: {s}"),
            Self::SeekNotSupported => write!(f, "Seek not supported for this source"),
            Self::SeekOutOfRange { pos_ms } => {
                write!(f, "Seek out of range: {pos_ms} ms")
            }
            Self::Pipeline(s) => write!(f, "Pipeline error: {s}"),
            Self::NoSource => write!(f, "No source loaded"),
            Self::Poisoned => write!(f, "Internal mutex poisoned"),
        }
    }
}

impl std::error::Error for AudioError {}

// ── From impls for ergonomic conversion ──────────────────────────────────────

impl From<std::io::Error> for AudioError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<cpal::SupportedStreamConfigsError> for AudioError {
    fn from(e: cpal::SupportedStreamConfigsError) -> Self {
        Self::StreamConfig(e.to_string())
    }
}

impl From<cpal::BuildStreamError> for AudioError {
    fn from(e: cpal::BuildStreamError) -> Self {
        Self::StreamBuild(e.to_string())
    }
}

impl From<cpal::DefaultStreamConfigError> for AudioError {
    fn from(e: cpal::DefaultStreamConfigError) -> Self {
        Self::StreamConfig(e.to_string())
    }
}

impl From<cpal::PlayStreamError> for AudioError {
    fn from(e: cpal::PlayStreamError) -> Self {
        Self::StreamBuild(e.to_string())
    }
}

/// Convert an `AudioError` to the legacy `String` type used in FFI helpers.
///
/// This allows existing `Result<T, AudioError>` values to be mapped with
/// `.map_err(|e| e.to_string())` when the FFI surface still requires `String`.
impl From<AudioError> for String {
    fn from(e: AudioError) -> Self {
        e.to_string()
    }
}
