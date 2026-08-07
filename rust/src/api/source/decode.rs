use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::{Receiver, Sender};
use flutter_rust_bridge::frb;
use symphonia::core::{
    audio::GenericAudioBufferRef,
    codecs::{CodecParameters, audio::AudioDecoderOptions},
    formats::{FormatOptions, TrackType, probe::Hint},
    io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions},
    meta::MetadataOptions,
};

use crate::{
    api::{
        enums::{DECODE_BACKPRESSURE_SLEEP_MS, MAX_RATE, MIN_RATE},
        renderer::{
            output::AudioOutputConfig,
            state::{ResampleState, SharedControlState},
        },
        source::{AudioSource, http_stream::HttpStream},
    },
    error, info, warn,
};

type BoxedMediaSource = Box<dyn symphonia::core::io::MediaSource>;

pub enum DecodeCommand {
    Seek { millis: i32 },
    Stop,
}

#[frb(opaque)]
pub struct DecodePool {
    pub(crate) command_tx: Option<Sender<DecodeCommand>>,
    pool: Option<JoinHandle<()>>,
    pub(crate) duration: i32,
    pub(crate) source: Option<AudioSource>,
}

impl DecodePool {
    pub(crate) fn new() -> DecodePool {
        DecodePool { command_tx: None, pool: None, duration: -1, source: None }
    }

    pub(crate) fn set_source(&mut self, source: AudioSource) {
        self.duration = estimate_duration_millis(&source).unwrap_or(-1);
        self.source = Some(source);
    }

    pub fn is_decoding(&self) -> bool {
        self.pool.as_ref().is_some_and(|h| !h.is_finished())
    }

    /// Drop handles for a decode thread that has already exited, or join one
    /// that was signaled to stop via [`signal_stop`].
    pub(crate) fn reap_finished(&mut self) {
        if self.command_tx.is_none() {
            if let Some(handle) = self.pool.take() {
                let _ = handle.join();
            }
            return;
        }
        if self.pool.as_ref().is_some_and(|h| h.is_finished()) {
            if let Some(handle) = self.pool.take() {
                let _ = handle.join();
            }
            self.command_tx = None;
        }
    }

    /// Start a new decode thread.  Each call creates a fresh command channel so
    /// there are never stale commands from a previous playback session.
    pub(crate) fn start(
        &mut self,
        ctrl: Arc<SharedControlState>,
        queue_prod: ringbuf::HeapProd<f32>,
        output_config: AudioOutputConfig,
        initial_millis: i32,
    ) -> Sender<DecodeCommand> {
        // If a thread is already running, just seek it.
        if self.command_tx.is_some() && self.is_decoding() {
            self.seek(initial_millis);
            return self.command_tx.as_ref().unwrap().clone();
        }

        let (tx, rx) = crossbeam_channel::unbounded::<DecodeCommand>();
        self.command_tx = Some(tx.clone());

        let source = match self.source.clone() {
            Some(s) => s,
            None => {
                warn!("Call set source first!");
                return tx;
            }
        };

        let thr_handle = thread::Builder::new()
            .name("Decode thread".to_string())
            .spawn(move || {
                if let Err(e) = decode_and_feed(
                    source,
                    ctrl,
                    queue_prod,
                    output_config.channels as usize,
                    output_config.sample_rate,
                    initial_millis,
                    rx,
                ) {
                    error!("Decode thread failed: {e}");
                }
            })
            .expect("Can not start decode thread");

        self.pool = Some(thr_handle);
        tx
    }

    /// Send a seek command to the running decode thread (no restart).
    pub fn seek(&self, millis: i32) {
        info!("Seek to {}", millis);
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(DecodeCommand::Seek { millis });
        }
    }

    /// Signal the decode thread to stop without blocking.
    pub(crate) fn signal_stop(&mut self) {
        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(DecodeCommand::Stop);
        }
    }

    /// Signal stop and wait for the decode thread to exit.
    pub(crate) fn stop(&mut self) {
        self.signal_stop();
        if let Some(handle) = self.pool.take() {
            let _ = handle.join();
        }
        self.pool = None;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// decode_and_feed — runs in the dedicated decode thread
// ─────────────────────────────────────────────────────────────────────────────

use symphonia::core::formats::SeekTo;
use symphonia::core::units::Time;

fn decode_and_feed(
    source: AudioSource,
    ctrl: Arc<SharedControlState>,
    mut queue_prod: ringbuf::HeapProd<f32>,
    out_channels: usize,
    out_sample_rate: u32,
    initial_millis: i32,
    command_rx: Receiver<DecodeCommand>,
) -> Result<(), String> {
    use ringbuf::traits::Producer;

    let media = media_source_from_owned(source)?;
    let mss = MediaSourceStream::new(media, MediaSourceStreamOptions::default());

    let mut probed = symphonia::default::get_probe()
        .probe(&Hint::new(), mss, FormatOptions::default(), MetadataOptions::default())
        .map_err(|e| format!("Failed to probe audio format: {e}"))?;

    let track = probed
        .tracks()
        .iter()
        .find(|v| v.codec_params.as_ref().map(|p| p.is_audio()).unwrap_or(false))
        .ok_or_else(|| "No audio track can be found".to_string())?;

    let track_id = track.id;

    let codec_params = track
        .codec_params
        .as_ref()
        .ok_or_else(|| "No codec available".to_string())?;

    let audio_params = codec_params
        .audio()
        .ok_or_else(|| "Invalid audio codec".to_string())?;

    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(audio_params, &AudioDecoderOptions::default())
        .map_err(|e| format!("Failed to create decoder: {e}"))?;

    let mut resample_state = ResampleState::new();

    ctrl.stream_ended.store(false, std::sync::atomic::Ordering::Release);

    // Seek to initial position (skip if initial_millis < 0, meaning "start of file").
    if initial_millis >= 0 {
        decoder.reset();
        resample_state.reset();
        let seek_time = Time::from_millis(initial_millis as i64);
        let _ = probed.seek(
            symphonia::core::formats::SeekMode::Accurate,
            SeekTo::Time { time: seek_time, track_id: Some(track_id) },
        );
    }

    info!("Start new decode thread");

    loop {
        let packet = match probed.next_packet() {
            Ok(Some(p)) => p,
            Ok(None) => {
                ctrl.stream_ended.store(true, std::sync::atomic::Ordering::Release);
                break Ok(());
            }
            Err(_) => {
                ctrl.stream_ended.store(true, std::sync::atomic::Ordering::Release);
                info!("End of file");
                break Ok(());
            }
        };

        let decoded = match decoder.decode(&packet) {
            Ok(b)  => b,
            Err(e) => break Err(format!("Failed to decode packet: {e}")),
        };

        let (src_ch, src_rate, interleaved) = decoded_to_interleaved_f32(decoded);
        if interleaved.is_empty() || src_ch == 0 || src_rate == 0 {
            info!("Interleaved is empty");
            continue;
        }

        // Read playback rate from atomic — zero mutex.
        let playback_rate = ctrl.rate().clamp(MIN_RATE, MAX_RATE);
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
            info!("out is empty");
            continue;
        }

        let mut offset = 0;
        while offset < out.len() {
            // Drain any pending commands before pushing samples.
            while let Ok(cmd) = command_rx.try_recv() {
                match cmd {
                    DecodeCommand::Seek { millis } => {
                        // AudioPlayer::seek() already drains the consumer
                        // and updates source_position before this message
                        // arrives.  We just need to reset the decoder and
                        // Symphonia reader, then skip remaining output from
                        // the old packet.
                        decoder.reset();
                        resample_state.reset();
                        ctrl.stream_ended.store(false, std::sync::atomic::Ordering::Release);
                        let seek_time = Time::from_millis(millis as i64);
                        let _ = probed.seek(
                            symphonia::core::formats::SeekMode::Accurate,
                            SeekTo::Time { time: seek_time, track_id: Some(track_id) },
                        );
                        offset = out.len(); // skip remaining output from old position
                    }
                    DecodeCommand::Stop => return Ok(()),
                }
            }

            if offset >= out.len() { break; }

            // Push as many samples as the ring buffer will accept (lock-free).
            let pushed = queue_prod.push_slice(&out[offset..]);
            if pushed == 0 {
                // Queue is full: yield briefly so the audio callback can drain.
                thread::sleep(Duration::from_millis(DECODE_BACKPRESSURE_SLEEP_MS));
            } else {
                offset += pushed;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Sample format conversion helpers
// ─────────────────────────────────────────────────────────────────────────────

fn decoded_to_interleaved_f32(decoded: GenericAudioBufferRef<'_>) -> (usize, u32, Vec<f32>) {
    let spec = decoded.spec();
    let channels = spec.channels().count();
    let rate = spec.rate();
    let mut samples = Vec::<f32>::new();
    decoded.copy_to_vec_interleaved(&mut samples);
    (channels, rate, samples)
}

#[inline(always)]
fn source_frame_sample(
    frames: &[f32],
    src_channels: usize,
    frame: usize,
    out_channel: usize,
    out_channels: usize,
) -> f32 {
    if src_channels == 0 { return 0.0; }
    let base = frame * src_channels;
    if out_channels == 1 && src_channels > 1 {
        let acc: f32 = (0..src_channels).map(|c| frames[base + c]).sum();
        return acc / src_channels as f32;
    }
    if src_channels == 1 { return frames[base]; }
    let idx = out_channel.min(src_channels - 1);
    frames[base + idx]
}

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

    let mut frames = Vec::with_capacity(state.carry.len() + src_interleaved.len());
    frames.extend_from_slice(&state.carry);
    frames.extend_from_slice(src_interleaved);

    let total_frames = frames.len() / src_channels;
    if total_frames < 2 { state.carry = frames; return Vec::new(); }

    let step = src_rate as f64 / out_rate as f64;
    let mut pos = state.pos;

    let estimated =
        (((total_frames as f64 - pos - 1.0) / step).max(0.0).ceil() as usize).saturating_add(1);
    let mut out = Vec::with_capacity(estimated.saturating_mul(out_channels));

    while pos + 1.0 < total_frames as f64 {
        let i0   = pos.floor() as usize;
        let frac = (pos - i0 as f64) as f32;
        for ch in 0..out_channels {
            let s0 = source_frame_sample(&frames, src_channels, i0,     ch, out_channels);
            let s1 = source_frame_sample(&frames, src_channels, i0 + 1, ch, out_channels);
            out.push(s0 + (s1 - s0) * frac);
        }
        pos += step;
    }

    let keep_frame = total_frames - 1;
    let keep_base = keep_frame * src_channels;
    state.carry.clear();
    state.carry.extend_from_slice(&frames[keep_base..keep_base + src_channels]);
    state.pos = pos - keep_frame as f64;
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// MediaSource helpers
// ─────────────────────────────────────────────────────────────────────────────

pub struct MemorySource<T: AsRef<[u8]>> { inner: Cursor<T> }

impl<T: AsRef<[u8]>> MemorySource<T> {
    pub fn new(data: T) -> Self { Self { inner: Cursor::new(data) } }
}
impl<T: AsRef<[u8]>> Read for MemorySource<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> { self.inner.read(buf) }
}
impl<T: AsRef<[u8]>> Seek for MemorySource<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> { self.inner.seek(pos) }
}
impl<T: AsRef<[u8]> + Send + Sync> MediaSource for MemorySource<T> {
    fn is_seekable(&self) -> bool { true }
    fn byte_len(&self) -> Option<u64> {
        Some(self.inner.get_ref().as_ref().len() as u64)
    }
}

fn media_source_from_owned(source: AudioSource) -> Result<BoxedMediaSource, String> {
    match source {
        AudioSource::Path(p) => {
            let f = File::open(&p).map_err(|e| format!("Failed to open file '{p}': {e}"))?;
            Ok(Box::new(f))
        }
        AudioSource::Url(u) => Ok(Box::new(HttpStream::new(&u)?)),
        AudioSource::Memory(data) => Ok(Box::new(MemorySource::new(data))),
    }
}

fn media_source_from_ref(source: &AudioSource) -> Result<BoxedMediaSource, String> {
    match source {
        AudioSource::Path(p) => {
            let f = File::open(p)
                .map_err(|e| format!("Failed to open source for duration: {e}"))?;
            Ok(Box::new(f))
        }
        AudioSource::Url(u) => Ok(Box::new(HttpStream::new(u)?)),
        AudioSource::Memory(data) => Ok(Box::new(MemorySource::new(data.clone()))),
    }
}

fn estimate_duration_millis(source: &AudioSource) -> Result<i32, String> {
    let media = match media_source_from_ref(source) {
        Ok(m)  => m,
        Err(e) => { crate::error!("{e}"); return Ok(-1); }
    };
    let mss = MediaSourceStream::new(media, MediaSourceStreamOptions::default());
    let probed = symphonia::default::get_probe()
        .probe(&Hint::new(), mss, FormatOptions::default(), MetadataOptions::default())
        .map_err(|e| format!("Failed to probe audio format: {e}"))?;

    let track = probed.default_track(TrackType::Audio);
    let Some(track) = track else { return Ok(-1) };
    let Some(CodecParameters::Audio(audio)) = track.codec_params.as_ref() else { return Ok(-1) };
    let Some(sr) = audio.sample_rate else { return Ok(-1) };
    if let Some(nf) = track.num_frames {
        Ok(((nf as f64 / sr as f64) * 1000.0) as i32)
    } else {
        Ok(-1)
    }
}
