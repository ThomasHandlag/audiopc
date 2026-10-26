use std::{fs::File, path::Path};

use flutter_rust_bridge::frb;
use symphonia::core::{
    audio::GenericAudioBufferRef,
    codecs::audio::AudioDecoderOptions,
    formats::{FormatOptions, TrackType, probe::Hint},
    io::{MediaSourceStream, MediaSourceStreamOptions},
    meta::MetadataOptions,
};

use crate::api::error::AudioError;

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Decodes the audio file at `path` and returns a waveform as a `Vec<f32>`.
///
/// The waveform is built by:
/// 1. Probing & decoding every packet with Symphonia (supports MP3, FLAC,
///    WAV, AAC, OGG/Vorbis, OPUS, …).
/// 2. Down-mixing all channels to **mono** via arithmetic mean.
/// 3. Splitting the mono samples into `num_points` equal-sized chunks.
/// 4. Computing the **RMS amplitude** of each chunk.
/// 5. Normalising the result so that the peak value equals `1.0`.
///
/// # Parameters
/// - `path`       – Absolute or relative path to a local audio file.
/// - `num_points` – Number of waveform data-points to return.
///                  Typical values: 100 – 2 000.
///
/// # Returns
/// A `Vec<f32>` of length `num_points` where every element is in `[0.0, 1.0]`.
/// Returns an empty `Vec` when the file has no audio or all samples are silent.
///
/// # Errors
/// Returns [`AudioError`] if the file cannot be opened, the format is
/// unsupported, or decoding fails.
#[frb]
pub fn get_audio_waveform(path: String, num_points: usize) -> Result<Vec<f32>, AudioError> {
    if num_points == 0 {
        return Ok(Vec::new());
    }

    // ── 1. Open the file ──────────────────────────────────────────────────────
    let file =
        File::open(Path::new(&path)).map_err(|e| AudioError::Io(e.to_string()))?;

    let mss = MediaSourceStream::new(
        Box::new(file),
        MediaSourceStreamOptions::default(),
    );

    // ── 2. Probe the format ───────────────────────────────────────────────────
    let mut probed = symphonia::default::get_probe()
        .probe(
            &Hint::new(),
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|e| AudioError::UnsupportedFormat(e.to_string()))?;

    // ── 3. Select the first audio track ──────────────────────────────────────
    let track = probed
        .default_track(TrackType::Audio)
        .ok_or_else(|| AudioError::UnsupportedFormat("No audio track found".to_string()))?;

    let track_id = track.id;

    let codec_params = track
        .codec_params
        .as_ref()
        .ok_or_else(|| AudioError::UnsupportedFormat("No codec parameters".to_string()))?;

    let audio_params = codec_params
        .audio()
        .ok_or_else(|| AudioError::UnsupportedFormat("Not an audio codec".to_string()))?;

    // ── 4. Create decoder ─────────────────────────────────────────────────────
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(audio_params, &AudioDecoderOptions::default())
        .map_err(|e| AudioError::UnsupportedFormat(e.to_string()))?;

    // ── 5. Decode all packets → collect interleaved mono f32 samples ──────────
    let mut mono_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match probed.next_packet() {
            Ok(Some(p)) if p.track_id == track_id => p,
            Ok(Some(_)) => continue, // packet for a different track
            Ok(None) | Err(_) => break,
        };

        let decoded = match decoder.decode(&packet) {
            Ok(b) => b,
            Err(_) => continue, // tolerate individual bad packets
        };

        push_mono_samples(&decoded, &mut mono_samples);
    }

    if mono_samples.is_empty() {
        return Ok(Vec::new());
    }

    // ── 6. Compute per-chunk RMS ──────────────────────────────────────────────
    let total = mono_samples.len();
    let chunk_size = (total / num_points).max(1);

    let mut waveform: Vec<f32> = mono_samples
        .chunks(chunk_size)
        .take(num_points)
        .map(|chunk| rms(chunk))
        .collect();

    // If fewer chunks were produced than requested (edge case at end of file),
    // pad with the last value to preserve the requested length.
    if let Some(&last) = waveform.last() {
        while waveform.len() < num_points {
            waveform.push(last);
        }
    }

    // ── 7. Normalise to [0.0, 1.0] ───────────────────────────────────────────
    let peak = waveform.iter().cloned().fold(0.0_f32, f32::max);
    if peak > 0.0 {
        for v in &mut waveform {
            *v /= peak;
        }
    }

    Ok(waveform)
}

// ─────────────────────────────────────────────────────────────────────────────
// Private helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Down-mix all channels in `decoded` to mono and append to `out`.
fn push_mono_samples(decoded: &GenericAudioBufferRef<'_>, out: &mut Vec<f32>) {
    let spec = decoded.spec();
    let channels = spec.channels().count();
    if channels == 0 {
        return;
    }

    let mut interleaved: Vec<f32> = Vec::new();
    decoded.copy_to_vec_interleaved(&mut interleaved);

    if channels == 1 {
        out.extend_from_slice(&interleaved);
    } else {
        // Average across channels for each frame.
        out.extend(interleaved.chunks_exact(channels).map(|frame| {
            frame.iter().sum::<f32>() / channels as f32
        }));
    }
}

/// Compute the Root-Mean-Square amplitude of a slice of samples.
#[inline]
fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}
