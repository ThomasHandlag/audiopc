use std::collections::VecDeque;
use std::ffi::CStr;
use std::fs::File;
use std::io::Cursor;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleFormat, SampleRate, Stream, StreamConfig};
use once_cell::sync::Lazy;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

const MAX_QUEUE_SECONDS: usize = 20;

static LAST_ERROR: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

fn set_last_error(msg: impl Into<String>) {
    if let Ok(mut last) = LAST_ERROR.lock() {
        *last = msg.into();
    }
}

fn clear_last_error() {
    if let Ok(mut last) = LAST_ERROR.lock() {
        last.clear();
    }
}

#[derive(Clone)]
enum AudioSource {
    Path(String),
    Url(String),
    Memory(Vec<u8>),
}

struct ResampleState {
    pos: f64,
    carry: Vec<f32>,
}

impl ResampleState {
    fn new() -> Self {
        Self {
            pos: 0.0,
            carry: Vec::new(),
        }
    }
}

struct SharedPlayback {
    queue: VecDeque<f32>,
    playing: bool,
    volume: f32,
    lowpass_hz: f32,
    lowpass_prev: Vec<f32>,
    channels: usize,
    sample_rate: u32,
}

impl SharedPlayback {
    fn new(channels: usize, sample_rate: u32) -> Self {
        Self {
            queue: VecDeque::new(),
            playing: false,
            volume: 1.0,
            lowpass_hz: 0.0,
            lowpass_prev: vec![0.0; channels],
            channels,
            sample_rate,
        }
    }

    fn clear_audio_state(&mut self) {
        self.queue.clear();
        self.lowpass_prev.fill(0.0);
    }

    fn max_samples(&self) -> usize {
        self.sample_rate
            .saturating_mul(self.channels as u32)
            .saturating_mul(MAX_QUEUE_SECONDS as u32) as usize
    }

    fn push_samples_bounded(&mut self, samples: &[f32]) -> usize {
        let available = self.max_samples().saturating_sub(self.queue.len());
        let push_count = samples.len().min(available);
        self.queue.extend(samples.iter().take(push_count).copied());
        push_count
    }

    fn next_sample(&mut self, channel: usize) -> f32 {
        if !self.playing {
            return 0.0;
        }

        let raw = self.queue.pop_front().unwrap_or(0.0);
        let mut sample = raw * self.volume;

        if self.lowpass_hz > 0.0 {
            let dt = 1.0 / self.sample_rate.max(1) as f32;
            let rc = 1.0 / (2.0 * std::f32::consts::PI * self.lowpass_hz.max(1.0));
            let alpha = dt / (rc + dt);
            let prev = self.lowpass_prev[channel];
            let filtered = prev + alpha * (sample - prev);
            self.lowpass_prev[channel] = filtered;
            sample = filtered;
        }

        sample.clamp(-1.0, 1.0)
    }
}

struct AudioEngine {
    shared: Arc<Mutex<SharedPlayback>>,
    stream_started: bool,
    source: Option<AudioSource>,
    decode_thread: Option<JoinHandle<()>>,
    decode_stop: Arc<AtomicBool>,
    out_channels: usize,
    out_sample_rate: u32,
}

impl AudioEngine {
    fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No default output device available".to_string())?;

        let config = device
            .default_output_config()
            .map_err(|e| format!("Failed to read default output config: {e}"))?;

        let out_channels = config.channels() as usize;
        let out_sample_rate = config.sample_rate().0;

        Ok(Self {
            shared: Arc::new(Mutex::new(SharedPlayback::new(
                out_channels,
                out_sample_rate,
            ))),
            stream_started: false,
            source: None,
            decode_thread: None,
            decode_stop: Arc::new(AtomicBool::new(false)),
            out_channels,
            out_sample_rate,
        })
    }

    fn ensure_stream(&mut self) -> Result<(), String> {
        if self.stream_started {
            return Ok(());
        }

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No default output device available".to_string())?;

        let output_config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default output config: {e}"))?;

        let sample_format = output_config.sample_format();
        let stream_config = StreamConfig {
            channels: output_config.channels(),
            sample_rate: SampleRate(self.out_sample_rate),
            buffer_size: BufferSize::Default,
        };

        let shared = Arc::clone(&self.shared);
        let channels = self.out_channels;

        let err_fn = |err| {
            eprintln!("CPAL stream error: {err}");
        };

        let stream = match sample_format {
            SampleFormat::F32 => device
                .build_output_stream(
                    &stream_config,
                    move |data: &mut [f32], _| write_output_f32(data, channels, &shared),
                    err_fn,
                    None,
                )
                .map_err(|e| format!("Failed to build f32 output stream: {e}"))?,
            SampleFormat::I16 => {
                let shared_i16 = Arc::clone(&self.shared);
                device
                    .build_output_stream(
                        &stream_config,
                        move |data: &mut [i16], _| write_output_i16(data, channels, &shared_i16),
                        err_fn,
                        None,
                    )
                    .map_err(|e| format!("Failed to build i16 output stream: {e}"))?
            }
            SampleFormat::U16 => {
                let shared_u16 = Arc::clone(&self.shared);
                device
                    .build_output_stream(
                        &stream_config,
                        move |data: &mut [u16], _| write_output_u16(data, channels, &shared_u16),
                        err_fn,
                        None,
                    )
                    .map_err(|e| format!("Failed to build u16 output stream: {e}"))?
            }
            _ => return Err("Unsupported sample format".to_string()),
        };

        stream
            .play()
            .map_err(|e| format!("Failed to start output stream: {e}"))?;

        // Keep the CPAL stream alive for the process lifetime.
        let _leaked: &'static mut Stream = Box::leak(Box::new(stream));
        self.stream_started = true;
        Ok(())
    }

    fn set_source(&mut self, source: AudioSource) {
        self.source = Some(source);
        self.stop_decode_thread();
        if let Ok(mut shared) = self.shared.lock() {
            shared.clear_audio_state();
        }
    }

    fn set_playing(&mut self, playing: bool) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.playing = playing;
        }
    }

    fn stop(&mut self) {
        self.set_playing(false);
        self.stop_decode_thread();
        if let Ok(mut shared) = self.shared.lock() {
            shared.clear_audio_state();
        }
    }

    fn set_volume(&mut self, volume: f32) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.volume = volume.clamp(0.0, 4.0);
        }
    }

    fn set_lowpass_hz(&mut self, hz: f32) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.lowpass_hz = hz.max(0.0);
            shared.lowpass_prev.fill(0.0);
        }
    }

    fn start_decode_thread_if_needed(&mut self) -> Result<(), String> {
        if self.decode_thread.is_some() {
            return Ok(());
        }

        let source = self
            .source
            .clone()
            .ok_or_else(|| "No source loaded. Call set_source first.".to_string())?;

        self.decode_stop.store(false, Ordering::SeqCst);
        let stop_flag = Arc::clone(&self.decode_stop);
        let shared = Arc::clone(&self.shared);
        let out_channels = self.out_channels;
        let out_sample_rate = self.out_sample_rate;

        let handle = thread::spawn(move || {
            if let Err(err) = decode_and_feed(source, stop_flag, shared, out_channels, out_sample_rate)
            {
                set_last_error(err.clone());
                eprintln!("Decode thread ended with error: {err}");
            }
        });

        self.decode_thread = Some(handle);
        Ok(())
    }

    fn stop_decode_thread(&mut self) {
        self.decode_stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.decode_thread.take() {
            let _ = handle.join();
        }
        self.decode_stop.store(false, Ordering::SeqCst);
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.stop();
    }
}

static ENGINE: Lazy<Mutex<Option<AudioEngine>>> = Lazy::new(|| Mutex::new(None));

fn with_engine_mut<F>(mut f: F) -> i32
where
    F: FnMut(&mut AudioEngine) -> Result<(), String>,
{
    let mut guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => {
            set_last_error("Engine mutex is poisoned");
            return -500;
        }
    };

    if guard.is_none() {
        match AudioEngine::new() {
            Ok(engine) => {
                *guard = Some(engine);
            }
            Err(err) => {
                set_last_error(err);
                return -501;
            }
        }
    }

    let Some(engine) = guard.as_mut() else {
        set_last_error("Engine initialization failed");
        return -502;
    };

    match f(engine) {
        Ok(()) => {
            clear_last_error();
            0
        }
        Err(err) => {
            set_last_error(err);
            -1
        }
    }
}

fn c_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    // SAFETY: Caller promises ptr is a valid, NUL-terminated C string.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().ok().map(ToOwned::to_owned)
}

fn decode_and_feed(
    source: AudioSource,
    stop_flag: Arc<AtomicBool>,
    shared: Arc<Mutex<SharedPlayback>>,
    out_channels: usize,
    out_sample_rate: u32,
) -> Result<(), String> {
    let media_source: Box<dyn symphonia::core::io::MediaSource> = match source {
        AudioSource::Path(path) => {
            let file = File::open(&path)
                .map_err(|e| format!("Failed to open file source '{path}': {e}"))?;
            Box::new(file)
        }
        AudioSource::Url(url) => {
            let response = reqwest::blocking::get(&url)
                .and_then(|res| res.error_for_status())
                .map_err(|e| format!("Failed to fetch URL source '{url}': {e}"))?;
            let bytes = response
                .bytes()
                .map_err(|e| format!("Failed to read response body: {e}"))?;
            Box::new(Cursor::new(bytes.to_vec()))
        },
        AudioSource::Memory(data) => Box::new(Cursor::new(data)),
    };

    let mss = MediaSourceStream::new(media_source, MediaSourceStreamOptions::default());

    let hint = Hint::new();
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Failed to probe audio format: {e}"))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "No decodable audio track found".to_string())?
        .clone();

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Failed to create decoder: {e}"))?;

    let mut resample_state = ResampleState::new();

    loop {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::ResetRequired) => {
                return Err("Decoder reset required and not supported".to_string())
            }
            Err(SymphoniaError::IoError(_)) => break,
            Err(err) => return Err(format!("Failed to read next packet: {err}")),
        };

        let decoded = match decoder.decode(&packet) {
            Ok(buf) => buf,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(_)) => break,
            Err(err) => return Err(format!("Failed to decode packet: {err}")),
        };

        let (src_channels, src_rate, mut interleaved) = decoded_to_interleaved_f32(decoded);
        if interleaved.is_empty() || src_channels == 0 || src_rate == 0 {
            continue;
        }

        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        let out = convert_to_output(
            &mut interleaved,
            src_channels,
            src_rate,
            out_channels,
            out_sample_rate,
            &mut resample_state,
        );

        if out.is_empty() {
            continue;
        }

        let mut offset = 0;
        while offset < out.len() {
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }

            let pushed = if let Ok(mut s) = shared.lock() {
                s.push_samples_bounded(&out[offset..])
            } else {
                0
            };

            if pushed == 0 {
                // Decoder runs faster than playback; wait for output callback to consume.
                thread::sleep(Duration::from_millis(5));
                continue;
            }

            offset += pushed;
        }
    }

    Ok(())
}

fn decoded_to_interleaved_f32(decoded: AudioBufferRef<'_>) -> (usize, u32, Vec<f32>) {
    let spec = *decoded.spec();
    let channels = spec.channels.count();
    let sample_rate = spec.rate;
    let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
    sample_buf.copy_interleaved_ref(decoded);
    (channels, sample_rate, sample_buf.samples().to_vec())
}

fn source_frame_sample(frames: &[f32], src_channels: usize, frame: usize, out_channel: usize, out_channels: usize) -> f32 {
    if src_channels == 0 {
        return 0.0;
    }

    let base = frame * src_channels;

    if out_channels == 1 && src_channels > 1 {
        let mut acc = 0.0;
        for c in 0..src_channels {
            acc += frames[base + c];
        }
        return acc / src_channels as f32;
    }

    if src_channels == 1 {
        return frames[base];
    }

    let idx = out_channel.min(src_channels - 1);
    frames[base + idx]
}

fn convert_to_output(
    src_interleaved: &mut [f32],
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
    if total_frames < 2 {
        state.carry = frames;
        return Vec::new();
    }

    let step = src_rate as f64 / out_rate as f64;
    let mut pos = state.pos;
    let mut out = Vec::new();

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

    let keep_frame = total_frames - 1;
    let keep_base = keep_frame * src_channels;
    state.carry = frames[keep_base..keep_base + src_channels].to_vec();
    state.pos = pos - keep_frame as f64;

    out
}

fn write_output_f32(data: &mut [f32], channels: usize, shared: &Arc<Mutex<SharedPlayback>>) {
    let mut guard = match shared.lock() {
        Ok(g) => g,
        Err(_) => {
            data.fill(0.0);
            return;
        }
    };

    for frame in data.chunks_mut(channels) {
        for (ch, out) in frame.iter_mut().enumerate() {
            *out = guard.next_sample(ch);
        }
    }
}

fn write_output_i16(data: &mut [i16], channels: usize, shared: &Arc<Mutex<SharedPlayback>>) {
    let mut guard = match shared.lock() {
        Ok(g) => g,
        Err(_) => {
            data.fill(0);
            return;
        }
    };

    for frame in data.chunks_mut(channels) {
        for (ch, out) in frame.iter_mut().enumerate() {
            let sample = guard.next_sample(ch);
            *out = (sample * i16::MAX as f32) as i16;
        }
    }
}

fn write_output_u16(data: &mut [u16], channels: usize, shared: &Arc<Mutex<SharedPlayback>>) {
    let mut guard = match shared.lock() {
        Ok(g) => g,
        Err(_) => {
            data.fill(u16::MAX / 2);
            return;
        }
    };

    for frame in data.chunks_mut(channels) {
        for (ch, out) in frame.iter_mut().enumerate() {
            let sample = guard.next_sample(ch);
            *out = (((sample * 0.5) + 0.5) * u16::MAX as f32) as u16;
        }
    }
}

#[no_mangle]
pub extern "C" fn audiopc_default_output_sample_rate() -> i32 {
    let host = cpal::default_host();
    let Some(device) = host.default_output_device() else {
        return -1;
    };

    match device.default_output_config() {
        Ok(config) => config.sample_rate().0 as i32,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn audiopc_default_output_channels() -> i32 {
    let host = cpal::default_host();
    let Some(device) = host.default_output_device() else {
        return -1;
    };

    match device.default_output_config() {
        Ok(config) => i32::from(config.channels()),
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn audiopc_output_device_count() -> i32 {
    let host = cpal::default_host();
    match host.output_devices() {
        Ok(devices) => devices.count() as i32,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn audiopc_set_source_path(path: *const c_char) -> i32 {
    let Some(path) = c_string(path) else {
        set_last_error("Source path is null or invalid UTF-8");
        return -2;
    };

    if File::open(&path).is_err() {
        set_last_error(format!("Could not open source file: {path}"));
        return -3;
    }

    with_engine_mut(|engine| {
        engine.set_source(AudioSource::Path(path.clone()));
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn audiopc_set_source_url(url: *const c_char) -> i32 {
    let Some(url) = c_string(url) else {
        set_last_error("Source URL is null or invalid UTF-8");
        return -2;
    };

    with_engine_mut(|engine| {
        engine.set_source(AudioSource::Url(url.clone()));
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn audiopc_set_source_memory(data: *const u8, len: i32) -> i32 {
    if data.is_null() || len <= 0 {
        set_last_error("Source memory pointer is null or length is non-positive");
        return -2;
    }

    let slice = unsafe { std::slice::from_raw_parts(data, len as usize) };
    let memory_data = Arc::new(Mutex::new(slice.to_vec()));

    with_engine_mut(|engine| {
        engine.set_source(AudioSource::Memory(memory_data.lock().unwrap().clone()));
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn audiopc_play() -> i32 {
    with_engine_mut(|engine| {
        engine.ensure_stream()?;
        engine.start_decode_thread_if_needed()?;
        engine.set_playing(true);
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn audiopc_pause() -> i32 {
    with_engine_mut(|engine| {
        engine.set_playing(false);
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn audiopc_stop() -> i32 {
    with_engine_mut(|engine| {
        engine.stop();
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn audiopc_set_volume(volume: f64) -> i32 {
    with_engine_mut(|engine| {
        engine.set_volume(volume as f32);
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn audiopc_set_lowpass_hz(hz: f64) -> i32 {
    with_engine_mut(|engine| {
        engine.set_lowpass_hz(hz as f32);
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn audiopc_buffered_samples() -> i32 {
    let guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };
    let Some(engine) = guard.as_ref() else {
        return 0;
    };
    let queued = match engine.shared.lock() {
        Ok(shared) => shared.queue.len() as i32,
        Err(_) => -1,
    };

    queued
}

#[no_mangle]
pub extern "C" fn audiopc_last_error_message_length() -> i32 {
    match LAST_ERROR.lock() {
        Ok(last) => last.len() as i32,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn audiopc_last_error_message_copy(buffer: *mut c_char, buffer_len: i32) -> i32 {
    if buffer.is_null() || buffer_len <= 0 {
        return -1;
    }

    let msg = match LAST_ERROR.lock() {
        Ok(last) => last.clone(),
        Err(_) => return -2,
    };

    let bytes = msg.as_bytes();
    let copy_len = bytes.len().min((buffer_len - 1) as usize);

    // SAFETY: `buffer` is expected to be valid for `buffer_len` bytes by caller.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, copy_len);
        *buffer.add(copy_len) = 0;
    }

    copy_len as i32
}
