use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::Receiver;
use flutter_rust_bridge::frb;
use parking_lot::Mutex;
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
        enums::{
            AudioCommand, DECODE_BACKPRESSURE_SLEEP_MS, DEFAULT_MAX_QUEUE_SECONDS,
            DEFAULT_VISUALIZER_SECONDS, MAX_RATE, MIN_RATE,
        },
        renderer::{
            output::{AudioOutput, AudioOutputConfig},
            state::{AudioState, BuffConfig, PlaybackState, ResampleState},
        },
        source::{AudioSource, http_stream::HttpStream},
    },
    error, info, warn,
};

type BoxedMediaSource = Box<dyn symphonia::core::io::MediaSource>;

use crossbeam_channel::Sender;

pub enum DecodeCommand {
    Seek { millis: i32 },
    Stop,
}

#[frb(opaque)]
pub struct DecodePool {
    pub(crate) command_tx: Option<Sender<DecodeCommand>>, // Replaces stop_flag
    pool: Option<JoinHandle<()>>,
    pub(crate) duration: i32,
    pub(crate) source: Option<AudioSource>,
}

impl DecodePool {
    pub(crate) fn new() -> DecodePool {
        DecodePool {
            command_tx: None,
            pool: None,
            duration: -1,
            source: None,
        }
    }

    pub(crate) fn set_source(&mut self, source: AudioSource) {
        self.duration = estimate_duration_millis(&source).unwrap_or(-1);
        self.source = Some(source);
    }

    pub fn is_decoding(&self) -> bool {
        self.pool.as_ref().is_some_and(|h| !h.is_finished())
    }

    /// Drop handles for a decode thread that has already exited.
    pub(crate) fn reap_finished(&mut self) {
        if self.pool.as_ref().is_some_and(|h| h.is_finished()) {
            if let Some(handle) = self.pool.take() {
                let _ = handle.join();
            }
            self.command_tx = None;
        }
    }

    /// Call this once when playback starts
    pub(crate) fn start(
        &mut self,
        output: Arc<Mutex<AudioOutput>>,
        state: Arc<Mutex<AudioState>>,
        output_config: AudioOutputConfig,
        initial_millis: i32,
        rc: Receiver<AudioCommand>,
    ) {
        // If already running, just seek instead of restarting
        if self.command_tx.is_some() && self.is_decoding() {
            self.seek(initial_millis);
            return;
        }

        self.reap_finished();

        let (tx, rx) = crossbeam_channel::bounded::<DecodeCommand>(16);
        let tx_clone = tx.clone();
        self.command_tx = Some(tx);

        let source = match self.source.clone() {
            Some(source) => source,
            None => {
                warn!("Call set source first!");
                return;
            }
        };

        let state_for_callback = Arc::clone(&state);

        let thr_handle = thread::Builder::new()
            .name("Decode thread".to_string())
            .spawn(move || {
                if let Err(e) = decode_and_feed(
                    source,
                    state,
                    output_config.channels as usize,
                    output_config.sample_rate,
                    initial_millis,
                    rx,
                    move || {
                        while let Ok(cmd) = rc.try_recv() {
                            match cmd {
                                AudioCommand::RebuildOuput => {
                                    state_for_callback.lock().pl_state = PlaybackState::Paused;

                                    let mut out = output.lock();
                                    out.dispose_old_stream();

                                    match out.build() {
                                        Ok(new_cfg) => {
                                            if new_cfg.sample_rate != output_config.sample_rate
                                                || new_cfg.channels != output_config.channels
                                            {
                                                let mut s = state_for_callback.lock();
                                                s.update_device_config(
                                                    BuffConfig::new(
                                                        (new_cfg.sample_rate as usize)
                                                            .saturating_mul(
                                                                new_cfg.channels as usize,
                                                            )
                                                            .saturating_mul(
                                                                DEFAULT_MAX_QUEUE_SECONDS,
                                                            ),
                                                        DEFAULT_MAX_QUEUE_SECONDS,
                                                        new_cfg.sample_rate,
                                                    ),
                                                    BuffConfig::new(
                                                        (new_cfg.sample_rate as usize)
                                                            .saturating_mul(
                                                                new_cfg.channels as usize,
                                                            )
                                                            .saturating_mul(
                                                                DEFAULT_VISUALIZER_SECONDS,
                                                            ),
                                                        DEFAULT_VISUALIZER_SECONDS,
                                                        new_cfg.sample_rate,
                                                    ),
                                                    new_cfg.channels as usize,
                                                );
                                            }
                                            state_for_callback.lock().pl_state =
                                                PlaybackState::Playing;

                                            let millis = state_for_callback.lock().position();

                                            let _ = tx_clone.send(DecodeCommand::Seek { millis });
                                        }
                                        Err(e) => {
                                            warn!("Failed to rebuild audio output: {e}");
                                        }
                                    }
                                }
                            }
                        }
                    },
                ) {
                    error!("Decode thread failed: {e}");
                }
            })
            .expect("Can not start decode thread");

        self.pool = Some(thr_handle);
    }

    /// Fast seek! No thread restart.
    pub fn seek(&self, millis: i32) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(DecodeCommand::Seek { millis });
        }
    }

    pub(crate) fn stop(&mut self) {
        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(DecodeCommand::Stop);
        }
        if let Some(handle) = self.pool.take() {
            let _ = handle.join();
        }
        self.pool = None;
    }
}
use symphonia::core::formats::SeekTo;
use symphonia::core::units::Time;

fn decode_and_feed<F>(
    source: AudioSource,
    shared: Arc<Mutex<AudioState>>,
    out_channels: usize,
    out_sample_rate: u32,
    initial_millis: i32,
    command_rx: Receiver<DecodeCommand>,
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

    let track_id = track.id;

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

    let mut resample_state = ResampleState::new();

    {
        let mut s = shared.lock();
        s.stream_ended = false;
    }

    if initial_millis > 0 {
        {
            let mut s = shared.lock();
            s.visualizer.clear();
            s.queue.clear();
        }

        decoder.reset();

        resample_state.reset();

        let seek_time = Time::from_millis(initial_millis as i64);
        let _ = probed.seek(
            symphonia::core::formats::SeekMode::Accurate,
            SeekTo::Time {
                time: seek_time,
                track_id: Some(track_id),
            },
        );
    }

    loop {
        f();

        let packet = match probed.next_packet() {
            Ok(Some(p)) => p,
            Ok(None) => {
                let mut s = shared.lock();
                s.stream_ended = true;
                break Ok(());
            }
            Err(_) => {
                // End of stream — signal the audio output to drain remaining
                // buffered samples before transitioning to Completed.
                let mut s = shared.lock();
                s.stream_ended = true;
                info!("End of file");
                break Ok(());
            }
        };

        let decoded = match decoder.decode(&packet) {
            Ok(b) => b,
            Err(e) => break Err(format!("Failed to decode packet: {e}")),
        };

        let (src_ch, src_rate, interleaved) = decoded_to_interleaved_f32(decoded);
        if interleaved.is_empty() || src_ch == 0 || src_rate == 0 {
            continue;
        }

        let playback_rate = shared.lock().pl_rate.clamp(MIN_RATE, MAX_RATE);

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

        let mut offset = 0;
        while offset < out.len() {
            while let Ok(cmd) = command_rx.try_recv() {
                match cmd {
                    DecodeCommand::Seek { millis } => {
                        {
                            let mut s = shared.lock();
                            s.visualizer.clear();
                            s.queue.clear();
                            s.stream_ended = false;
                        }

                        decoder.reset();

                        resample_state.reset();

                        let seek_time = Time::from_millis(millis as i64);
                        let _ = probed.seek(
                            symphonia::core::formats::SeekMode::Accurate,
                            SeekTo::Time {
                                time: seek_time,
                                track_id: Some(track_id),
                            },
                        );
                    }
                    DecodeCommand::Stop => return Ok(()),
                }
            }

            let pushed = {
                let mut s = shared.lock();
                s.push_samples_bounded(&out[offset..])
            };

            if pushed == 0 {
                thread::sleep(Duration::from_millis(DECODE_BACKPRESSURE_SLEEP_MS));
            } else {
                offset += pushed;
            }
        }
    }
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
