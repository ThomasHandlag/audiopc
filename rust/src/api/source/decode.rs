use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use flutter_rust_bridge::frb;
use symphonia::core::{
    audio::GenericAudioBufferRef,
    codecs::{CodecParameters, audio::AudioDecoderOptions},
    formats::{FormatOptions, TrackType, probe::Hint},
    io::{MediaSourceStream, MediaSourceStreamOptions},
    meta::MetadataOptions,
};

use tempfile::tempfile;

use crate::{
    api::{
        enums::{DECODE_BACKPRESSURE_SLEEP_MS, MAX_RATE, MIN_RATE},
        renderer::{
            output::AudioOuputConfig,
            state::{AudioState, ResampleState},
        },
        source::{AudioSource, http_stream::HttpStream},
    },
    error,
};

type BoxedMediaSource = Box<dyn symphonia::core::io::MediaSource>;

#[frb(opaque)]
pub struct DecodePool {
    stop_flag: Arc<AtomicBool>,
    pool: Option<JoinHandle<()>>,
    pub(crate) duration: i32,
    pub(crate) source: Option<AudioSource>,
}

impl DecodePool {
    pub(crate) fn new() -> DecodePool {
        DecodePool {
            pool: None,
            stop_flag: Arc::new(AtomicBool::new(true)),
            duration: -1,
            source: None,
        }
    }

    pub(crate) fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(handle) = self.pool.take() {
            let _ = handle.join();
        }
        self.stop_flag.store(false, Ordering::SeqCst);
    }

    pub(crate) fn set_source(&mut self, source: AudioSource) {
        self.source = Some(source);
    }

    pub(crate) fn build<F>(
        &mut self,
        f: F,
        state: Arc<Mutex<AudioState>>,
        output_config: AudioOuputConfig,
    ) -> JoinHandle<()>
    where
        F: Fn() + Send + 'static,
    {
        let stop_flag = Arc::clone(&self.stop_flag);

        let source = self.source.clone().expect("Source is empty!");

        self.duration = estimate_duration_millis(&source).unwrap_or(-1);

        thread::Builder::new()
            .name("Decode thread".to_string())
            .spawn(move || {
                let _ = decode_and_feed(
                    source,
                    stop_flag,
                    state,
                    output_config.channels as usize,
                    output_config.sample_rate,
                    f,
                );
            })
            .expect("Can not start decode thread")
    }
}

fn decode_and_feed<F>(
    source: AudioSource,
    stop_flag: Arc<AtomicBool>,
    shared: Arc<Mutex<AudioState>>,
    out_channels: usize,
    out_sample_rate: u32,
    f: F,
) -> Result<(), String>
where
    F: Fn() + Send + 'static,
{
    let media = media_source_from_owned(source)?;
    let mss = MediaSourceStream::new(media, MediaSourceStreamOptions::default());

    let mut probed = symphonia::default::get_probe()
        .probe(
            &Hint::new(),
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|e| format!("Failed to probe audio format: {e}"))?;

    let track = probed
        .tracks()
        .iter()
        .find(|v| {
            v.codec_params
                .as_ref()
                .expect("No codec available")
                .is_audio()
        })
        .expect("No audio track can be found");

    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(
            track
                .codec_params
                .as_ref()
                .expect("No codec found")
                .audio()
                .expect("Invalid audio codec"),
            &AudioDecoderOptions::default(),
        )
        .map_err(|e| format!("Failed to create decoder: {e}"))?;

    let initial_playback_rate = shared
        .lock()
        .map(|s| s.pl_rate.clamp(MIN_RATE, MAX_RATE))
        .unwrap_or(1.0);

    let start_millis = shared.lock().map(|v| v.start_millies).unwrap_or(0);

    let mut resample_state = ResampleState::new();
    let mut skip_output_samples = source_millis_to_output_samples(
        start_millis,
        out_sample_rate,
        out_channels,
        initial_playback_rate,
    );

    let decode_result = loop {
        if stop_flag.load(Ordering::SeqCst) {
            break Ok(());
        }

        f();

        let packet = match probed.next_packet() {
            Ok(p) => p,
            Err(e) => break Err(format!("Failed to read next packet: {e}")),
        }
        .unwrap();

        let decoded = match decoder.decode(&packet) {
            Ok(b) => b,
            Err(e) => break Err(format!("Failed to decode packet: {e}")),
        };

        let (src_ch, src_rate, interleaved) = decoded_to_interleaved_f32(decoded);
        if interleaved.is_empty() || src_ch == 0 || src_rate == 0 {
            continue;
        }

        if stop_flag.load(Ordering::SeqCst) {
            break Ok(());
        }

        let playback_rate = shared
            .lock()
            .map(|s| s.pl_rate.clamp(MIN_RATE, MAX_RATE))
            .unwrap_or(1.0);

        // Scale the target rate to implement speed without pitch shift.
        let effective_out_rate = ((out_sample_rate as f32) / playback_rate).max(1.0) as u32;

        let out = convert_to_output(
            &interleaved,
            src_ch,
            src_rate,
            out_channels,
            effective_out_rate,
            &mut resample_state,
        );

        if out.is_empty() {
            continue;
        }

        let mut start = 0usize;
        if skip_output_samples > 0 {
            let consumed = skip_output_samples.min(out.len());
            skip_output_samples -= consumed;
            start = consumed;
        }
        if start >= out.len() {
            continue;
        }

        let out_slice = &out[start..];
        let mut offset = 0;

        while offset < out_slice.len() {
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }

            let pushed = shared
                .lock()
                .map(|mut s| s.push_samples_bounded(&out_slice[offset..]))
                .unwrap_or(0);

            if pushed == 0 {
                thread::sleep(Duration::from_millis(DECODE_BACKPRESSURE_SLEEP_MS));
            } else {
                offset += pushed;
            }
        }
    };

    if let Ok(mut s) = shared.lock() {
        s.stream_ended = true;
    }

    decode_result
}

/// Convert a source-time offset into the number of output samples to skip.
fn source_millis_to_output_samples(
    start_millis: i32,
    out_sample_rate: u32,
    out_channels: usize,
    playback_rate: f32,
) -> usize {
    let playback_rate = playback_rate.clamp(MIN_RATE, MAX_RATE).max(1.0e-6);
    ((start_millis.max(0) as f64) * out_sample_rate as f64 * out_channels as f64
        / 1000.0
        / playback_rate as f64) as usize
}

// ── Sample format conversion helpers ─────────────────────────────────────────

/// Convert a Symphonia decoded buffer to interleaved `f32`.

fn decoded_to_interleaved_f32(decoded: GenericAudioBufferRef<'_>) -> (usize, u32, Vec<f32>) {
    let spec = decoded.spec();
    let channels = spec.channels().count();
    let rate = spec.rate();

    let mut samples = Vec::<f32>::new();
    decoded.copy_to_vec_interleaved(&mut samples);

    (channels, rate, samples)
}

/// Select the correct source sample for a given output channel.
///
/// Handles mono→stereo up-mix, stereo→mono down-mix, and channel remapping.
#[inline(always)]
fn source_frame_sample(
    frames: &[f32],
    src_channels: usize,
    frame: usize,
    out_channel: usize,
    out_channels: usize,
) -> f32 {
    if src_channels == 0 {
        return 0.0;
    }
    let base = frame * src_channels;

    // Stereo → mono.
    if out_channels == 1 && src_channels > 1 {
        let acc: f32 = (0..src_channels).map(|c| frames[base + c]).sum();
        return acc / src_channels as f32;
    }

    // Mono → any.
    if src_channels == 1 {
        return frames[base];
    }

    // Channel clip.
    let idx = out_channel.min(src_channels - 1);
    frames[base + idx]
}

/// Resample `src_interleaved` from `src_rate` to `out_rate` using linear
/// interpolation, remapping from `src_channels` to `out_channels`.
///
/// Fractional position and boundary carry samples are threaded through
/// `state` across calls so there are no inter-packet discontinuities.
fn convert_to_output(
    src_interleaved: &[f32],
    src_channels: usize,
    src_rate: u32,
    out_channels: usize,
    out_rate: u32,
    state: &mut ResampleState,
) -> Vec<f32> {
    if src_channels == 0 || out_channels == 0 || src_rate == 0 || out_rate == 0 {
        return Vec::new();
    }

    // Prepend the carry frame from the previous packet.
    let mut frames = Vec::with_capacity(state.carry.len() + src_interleaved.len());
    frames.extend_from_slice(&state.carry);
    frames.extend_from_slice(src_interleaved);

    let total_frames = frames.len() / src_channels;
    if total_frames < 2 {
        state.carry = frames;
        return Vec::new();
    }

    let step = src_rate as f64 / out_rate as f64;
    let mut pos = state.pos;

    let estimated =
        (((total_frames as f64 - pos - 1.0) / step).max(0.0).ceil() as usize).saturating_add(1);
    let mut out = Vec::with_capacity(estimated.saturating_mul(out_channels));

    while pos + 1.0 < total_frames as f64 {
        let i0 = pos.floor() as usize;
        let frac = (pos - i0 as f64) as f32;

        for ch in 0..out_channels {
            let s0 = source_frame_sample(&frames, src_channels, i0, ch, out_channels);
            let s1 = source_frame_sample(&frames, src_channels, i0 + 1, ch, out_channels);
            out.push(s0 + (s1 - s0) * frac);
        }

        pos += step;
    }

    // Keep the last source frame as the left neighbour for the next packet.
    let keep_frame = total_frames - 1;
    let keep_base = keep_frame * src_channels;
    state.carry.clear();
    state
        .carry
        .extend_from_slice(&frames[keep_base..keep_base + src_channels]);
    state.pos = pos - keep_frame as f64;

    out
}

/// Build a `BoxedMediaSource` from an owned [`AudioSource`].
fn media_source_from_owned(source: AudioSource) -> Result<BoxedMediaSource, String> {
    match source {
        AudioSource::Path(p) => {
            let f = File::open(&p).map_err(|e| format!("Failed to open file '{p}': {e}"))?;
            Ok(Box::new(f))
        }
        AudioSource::Url(u) => {
            let s = HttpStream::new(&u)?;
            Ok(Box::new(s))
        }
        AudioSource::Memory(data) => {
            let f = write_bytes_to_temp_file(&data, "memory")?;
            Ok(Box::new(f))
        }
    }
}

/// Build a `BoxedMediaSource` from a reference, avoiding cloning large
/// in-memory payloads (re-opens the file / connection instead).
fn media_source_from_ref(source: &AudioSource) -> Result<BoxedMediaSource, String> {
    match source {
        AudioSource::Path(p) => {
            let f =
                File::open(p).map_err(|e| format!("Failed to open source for duration: {e}"))?;
            Ok(Box::new(f))
        }
        AudioSource::Url(u) => {
            let s = HttpStream::new(u)?;
            Ok(Box::new(s))
        }
        AudioSource::Memory(data) => {
            let f = write_bytes_to_temp_file(data, "duration_memory")?;
            Ok(Box::new(f))
        }
    }
}

/// Write `bytes` to an anonymous temporary file and rewind to the start.
fn write_bytes_to_temp_file(bytes: &[u8], tag: &str) -> Result<File, String> {
    let mut file =
        tempfile().map_err(|e| format!("Failed to create temporary file for {tag}: {e}"))?;
    file.write_all(bytes)
        .map_err(|e| format!("Failed to write temporary file for {tag}: {e}"))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to rewind temporary file for {tag}: {e}"))?;
    Ok(file)
}

/// Probe a source for duration without decoding.
/// Returns `-1` if the duration cannot be determined.
fn estimate_duration_millis(source: &AudioSource) -> Result<i32, String> {
    let media = match media_source_from_ref(source) {
        Ok(m) => m,
        Err(e) => {
            error!("{e}");
            return Ok(-1);
        }
    };

    let mss = MediaSourceStream::new(media, MediaSourceStreamOptions::default());

    let probed = symphonia::default::get_probe()
        .probe(
            &Hint::new(),
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|e| format!("Failed to probe audio format: {e}"))?;

    let track = probed.default_track(TrackType::Audio);

    let Some(track) = track else { return Ok(-1) };

    let Some(CodecParameters::Audio(audio)) = track.codec_params.as_ref() else {
        return Ok(-1);
    };

    let Some(sr) = audio.sample_rate else {
        return Ok(-1);
    };

    if let Some(nf) = track.num_frames {
        Ok(((nf as f64 / sr as f64) * 1000.0) as i32)
    } else {
        Ok(-1)
    }
}
