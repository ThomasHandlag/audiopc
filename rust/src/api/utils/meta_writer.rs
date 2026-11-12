use flutter_rust_bridge::frb;
use lofty::{
    file::TaggedFileExt,
    picture::{MimeType, Picture, PictureType},
    prelude::*,
    probe::Probe,
    tag::{ItemKey, Tag, TagType},
};

use crate::api::error::AudioError;

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────

/// Fields to write (or update) in an audio file's tag.
///
/// Every field is `Option`; `None` means "leave unchanged".
/// Pass an empty `Vec` / `None` to clear artwork.
#[frb]
pub struct MetadataUpdate {
    /// Track title to set, or `None` to leave unchanged.
    pub title: Option<String>,
    /// Artist name to set, or `None` to leave unchanged.
    pub artist: Option<String>,
    /// Album name to set, or `None` to leave unchanged.
    pub album: Option<String>,
    /// Track number string (e.g. `"3"` or `"3/12"`), or `None` to leave
    /// unchanged.
    pub track_number: Option<String>,
    /// Release year as a four-digit string, or `None` to leave unchanged.
    pub year: Option<String>,
    /// Genre label, or `None` to leave unchanged.
    pub genre: Option<String>,
    /// Raw image bytes to embed as "Cover (Front)" artwork.
    ///
    /// - `Some(bytes)` → replaces any existing cover-front picture.
    /// - `None` → artwork is left unchanged.
    ///
    /// Pass `Some(Vec::new())` to explicitly remove the cover artwork.
    pub artwork_data: Option<Vec<u8>>,
    /// MIME type of the artwork bytes (e.g. `"image/jpeg"` or `"image/png"`).
    ///
    /// Required (and used) only when `artwork_data` is `Some` and non-empty.
    /// Defaults to `"image/jpeg"` when `None` but artwork bytes are supplied.
    pub artwork_mime_type: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Writes (or updates) metadata fields in the audio file at `path`.
///
/// Only the fields in `update` that are `Some(…)` are touched; all others
/// retain their current values in the file.
///
/// Supported formats and their native tag types:
/// | Format               | Tag type written        |
/// |----------------------|-------------------------|
/// | MP3                  | ID3v2                   |
/// | FLAC                 | Vorbis Comments         |
/// | OGG (Vorbis / Opus)  | Vorbis Comments         |
/// | WAV                  | ID3v2 (RIFF INFO chunk) |
/// | AIFF                 | ID3v2                   |
/// | MP4 / M4A            | MP4 iTunes atoms        |
///
/// # Parameters
/// - `path`   – Absolute or relative path to the audio file (modified in-place).
/// - `update` – The fields to write.
///
/// # Errors
/// Returns [`AudioError`] if the file cannot be opened, the format is
/// unsupported, or the write fails.
#[frb]
pub fn write_metadata(path: String, update: MetadataUpdate) -> Result<(), AudioError> {
    // ── 1. Open & probe the file ──────────────────────────────────────────────
    let mut tagged_file = Probe::open(&path)
        .map_err(|e| AudioError::Io(e.to_string()))?
        .guess_file_type()
        .map_err(|e| AudioError::Io(e.to_string()))?
        .read()
        .map_err(|e| AudioError::UnsupportedFormat(e.to_string()))?;

    // ── 2. Obtain (or create) the primary tag ─────────────────────────────────
    if tagged_file.primary_tag().is_none() {
        // Insert a sensible default tag type for the detected file kind.
        let tag_type = default_tag_type(tagged_file.file_type());
        tagged_file.insert_tag(Tag::new(tag_type));
    }

    let tag = tagged_file
        .primary_tag_mut()
        .ok_or_else(|| AudioError::UnsupportedFormat("No writable tag found".to_string()))?;

    // ── 3. Apply the requested field changes ──────────────────────────────────
    if let Some(title) = update.title {
        tag.set_title(title);
    }
    if let Some(artist) = update.artist {
        tag.set_artist(artist);
    }
    if let Some(album) = update.album {
        tag.set_album(album);
    }
    if let Some(track_number) = update.track_number {
        tag.insert_text(ItemKey::TrackNumber, track_number);
    }
    if let Some(year) = update.year {
        tag.insert_text(ItemKey::Year, year);
    }
    if let Some(genre) = update.genre {
        tag.set_genre(genre);
    }

    // ── 4. Handle artwork ─────────────────────────────────────────────────────
    if let Some(artwork_bytes) = update.artwork_data {
        // Remove any existing cover-front picture first.
        tag.remove_picture_type(PictureType::CoverFront);

        if !artwork_bytes.is_empty() {
            let mime = parse_mime(update.artwork_mime_type.as_deref());
            let picture = Picture::new_unchecked(
                PictureType::CoverFront,
                Some(mime),
                None, // description
                artwork_bytes,
            );
            tag.push_picture(picture);
        }
    }

    // ── 5. Save back to disk ──────────────────────────────────────────────────
    tag.save_to_path(&path, lofty::config::WriteOptions::default())
        .map_err(|e| AudioError::Io(e.to_string()))?;

    Ok(())
}

/// Writes **only** the title field to the audio file at `path`.
///
/// All other existing tag fields are preserved.
///
/// # Errors
/// Returns [`AudioError`] if the file cannot be opened, the format is
/// unsupported, or the write fails.
#[frb]
pub fn write_title(path: String, title: String) -> Result<(), AudioError> {
    write_metadata(
        path,
        MetadataUpdate {
            title: Some(title),
            artist: None,
            album: None,
            track_number: None,
            year: None,
            genre: None,
            artwork_data: None,
            artwork_mime_type: None,
        },
    )
}

/// Writes **only** the artist field to the audio file at `path`.
///
/// All other existing tag fields are preserved.
///
/// # Errors
/// Returns [`AudioError`] if the file cannot be opened, the format is
/// unsupported, or the write fails.
#[frb]
pub fn write_artist(path: String, artist: String) -> Result<(), AudioError> {
    write_metadata(
        path,
        MetadataUpdate {
            title: None,
            artist: Some(artist),
            album: None,
            track_number: None,
            year: None,
            genre: None,
            artwork_data: None,
            artwork_mime_type: None,
        },
    )
}

/// Embeds `artwork_bytes` as the "Cover (Front)" picture in the audio file at
/// `path`, replacing any previously stored cover.
///
/// Pass an empty `artwork_bytes` to remove the existing cover artwork.
///
/// # Parameters
/// - `path`           – Absolute or relative path to the audio file (modified
///                      in-place).
/// - `artwork_bytes`  – Raw image bytes (JPEG, PNG, …).
/// - `mime_type`      – MIME type string (e.g. `"image/jpeg"`).
///                      Defaults to `"image/jpeg"` when `None`.
///
/// # Errors
/// Returns [`AudioError`] if the file cannot be opened, the format is
/// unsupported, or the write fails.
#[frb]
pub fn write_artwork(
    path: String,
    artwork_bytes: Vec<u8>,
    mime_type: Option<String>,
) -> Result<(), AudioError> {
    write_metadata(
        path,
        MetadataUpdate {
            title: None,
            artist: None,
            album: None,
            track_number: None,
            year: None,
            genre: None,
            artwork_data: Some(artwork_bytes),
            artwork_mime_type: mime_type,
        },
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Private helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the preferred [`TagType`] for `file_type`.
fn default_tag_type(file_type: lofty::file::FileType) -> TagType {
    use lofty::file::FileType;
    match file_type {
        FileType::Flac | FileType::Vorbis | FileType::Opus | FileType::Speex => {
            TagType::VorbisComments
        }
        FileType::Mp4 => TagType::Mp4Ilst,
        // MP3, WAV, AIFF, and everything else → ID3v2
        _ => TagType::Id3v2,
    }
}

/// Parses a MIME-type string into a [`MimeType`].
///
/// Falls back to `MimeType::Jpeg` for unknown or absent strings.
fn parse_mime(mime: Option<&str>) -> MimeType {
    match mime {
        Some(s) => MimeType::from_str(s),
        None => MimeType::Jpeg,
    }
}
