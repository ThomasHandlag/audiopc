use flutter_rust_bridge::frb;
use lofty::{
    file::TaggedFileExt,
    prelude::*,
    probe::Probe,
    tag::ItemKey,
};

use crate::api::error::AudioError;

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────

/// Metadata read from an audio file.
///
/// All fields are `Option` – if the file or its tag does not carry a value the
/// field is `None` rather than an empty string.
#[frb]
pub struct AudioMetadata {
    /// Track title.
    pub title: Option<String>,
    /// Primary performing artist.
    pub artist: Option<String>,
    /// Album name.
    pub album: Option<String>,
    /// Track number (e.g. `"3"` or `"3/12"`).
    pub track_number: Option<String>,
    /// Release year as a four-digit string.
    pub year: Option<String>,
    /// Genre label.
    pub genre: Option<String>,
    /// Raw bytes of the first embedded artwork image found in the file.
    ///
    /// The bytes are in whatever format is stored in the tag (JPEG, PNG, …).
    /// Inspect the leading magic bytes or use the companion
    /// [`artwork_mime_type`] field to determine the format.
    pub artwork_data: Option<Vec<u8>>,
    /// MIME type of the artwork image (e.g. `"image/jpeg"`, `"image/png"`).
    pub artwork_mime_type: Option<String>,
    /// Total duration of the audio in milliseconds.
    pub duration_ms: Option<u64>,
    /// Audio bitrate in kbps.
    pub bitrate_kbps: Option<u32>,
    /// Sample rate in Hz.
    pub sample_rate_hz: Option<u32>,
    /// Number of audio channels.
    pub channels: Option<u8>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Reads all supported metadata from the audio file at `path`.
///
/// Supports MP3 (ID3v1/v2, APE), FLAC (Vorbis Comments), OGG/Vorbis, Opus,
/// WAV, AIFF, MP4/M4A, and more via the [`lofty`] crate.
///
/// # Parameters
/// - `path` – Absolute or relative path to a local audio file.
///
/// # Returns
/// An [`AudioMetadata`] struct where every field is `Some(…)` when the
/// corresponding tag is present, or `None` when absent.
///
/// # Errors
/// Returns [`AudioError`] if the file cannot be opened or the format is
/// unsupported.
#[frb]
pub fn read_metadata(path: String) -> Result<AudioMetadata, AudioError> {
    let tagged_file = Probe::open(&path)
        .map_err(|e| AudioError::Io(e.to_string()))?
        .guess_file_type()
        .map_err(|e| AudioError::Io(e.to_string()))?
        .read()
        .map_err(|e| AudioError::UnsupportedFormat(e.to_string()))?;

    // Prefer the primary tag; fall back to the first available tag.
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let (title, artist, album, track_number, year, genre, artwork_data, artwork_mime_type) =
        if let Some(t) = tag {
            let title = t.title().map(|s| s.into_owned());
            let artist = t.artist().map(|s| s.into_owned());
            let album = t.album().map(|s| s.into_owned());
            let track_number = t
                .get_string(&ItemKey::TrackNumber)
                .map(|s| s.to_owned());
            let year = t.get_string(&ItemKey::Year).map(|s| s.to_owned());
            let genre = t.genre().map(|s| s.into_owned());

            // Extract the first "Cover (Front)" picture, or any picture if none
            // is explicitly typed as a cover.
            let (artwork_data, artwork_mime_type) = extract_artwork(t);

            (
                title,
                artist,
                album,
                track_number,
                year,
                genre,
                artwork_data,
                artwork_mime_type,
            )
        } else {
            (None, None, None, None, None, None, None, None)
        };

    // Audio properties (duration, bitrate, …) are independent of tags.
    let props = tagged_file.properties();
    let duration_ms = Some(props.duration().as_millis() as u64);
    let bitrate_kbps = props.overall_bitrate();
    let sample_rate_hz = props.sample_rate();
    let channels = props.channels();

    Ok(AudioMetadata {
        title,
        artist,
        album,
        track_number,
        year,
        genre,
        artwork_data,
        artwork_mime_type,
        duration_ms,
        bitrate_kbps,
        sample_rate_hz,
        channels,
    })
}

/// Reads **only** the embedded artwork from the audio file at `path`.
///
/// This is more efficient than [`read_metadata`] when you only need the cover
/// image, because it skips decoding the other tag fields.
///
/// # Returns
/// `Some(bytes)` when artwork is found, or `None`.
///
/// # Errors
/// Returns [`AudioError`] if the file cannot be opened or the format is
/// unsupported.
#[frb]
pub fn read_artwork(path: String) -> Result<Option<Vec<u8>>, AudioError> {
    let tagged_file = Probe::open(&path)
        .map_err(|e| AudioError::Io(e.to_string()))?
        .guess_file_type()
        .map_err(|e| AudioError::Io(e.to_string()))?
        .read()
        .map_err(|e| AudioError::UnsupportedFormat(e.to_string()))?;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    Ok(tag.and_then(|t| extract_artwork(t).0))
}

// ─────────────────────────────────────────────────────────────────────────────
// Private helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Returns `(raw_bytes, mime_type)` for the best artwork picture in `tag`.
///
/// Preference order:
/// 1. `PictureType::CoverFront`
/// 2. Any other picture type
/// 3. `(None, None)` when no pictures are present
fn extract_artwork(tag: &lofty::tag::Tag) -> (Option<Vec<u8>>, Option<String>) {
    use lofty::picture::PictureType;

    let pictures = tag.pictures();
    if pictures.is_empty() {
        return (None, None);
    }

    // Prefer cover-front; otherwise take the first picture.
    let pic = pictures
        .iter()
        .find(|p| p.pic_type() == PictureType::CoverFront)
        .or_else(|| pictures.first())
        .expect("non-empty slice always has a first element");

    let mime = pic.mime_type().map(|m| m.to_string());
    (Some(pic.data().to_vec()), mime)
}
