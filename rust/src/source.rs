/// Typed audio source.
///
/// Every source variant must be convertible into a Symphonia `MediaSource`
/// (see `engine::media_source_from_owned` / `media_source_from_ref`).  The
/// enum is intentionally **small** — it carries only the *address* of the
/// data, not the decoded samples.
///
/// New source kinds can be added here without changing call sites, because
/// `engine::media_source_from_owned` is the single point of dispatch.
#[derive(Clone)]
pub enum AudioSource {
    /// A path to a local file (absolute or relative).
    Path(String),

    /// An HTTP/HTTPS URL.  The engine opens a streaming connection on first
    /// use via [`crate::http_stream::HttpStream`].
    Url(String),

    /// Raw bytes already loaded into memory (e.g., loaded from an asset
    /// bundle or received over IPC).  Written to a temporary file before
    /// handing to Symphonia so that seeking works correctly.
    Memory(Vec<u8>),
}

impl AudioSource {
    /// Returns `true` if the source is a network URL.
    pub fn is_remote(&self) -> bool {
        matches!(self, Self::Url(_))
    }

    /// Returns a human-readable description, suitable for logging.
    pub fn description(&self) -> String {
        match self {
            Self::Path(p) => format!("file://{p}"),
            Self::Url(u) => u.clone(),
            Self::Memory(b) => format!("<memory {} bytes>", b.len()),
        }
    }
}