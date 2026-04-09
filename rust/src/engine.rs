use std::collections::VecDeque;
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleFormat, Stream, StreamConfig};
use log::error;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::{Fft, FftPlanner};
use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

const DEFAULT_MAX_QUEUE_SECONDS: usize = 20;
const MIN_MAX_QUEUE_SECONDS: usize = 1;
const MAX_MAX_QUEUE_SECONDS: usize = 120;
const DECODE_BACKPRESSURE_SLEEP_MS: u64 = 2;
const DEFAULT_VISUALIZER_SECONDS: usize = 2;
const VISUALIZER_FFT_SIZE: usize = 1024;
const DEFAULT_VISUALIZER_BAR_COUNT: usize = 64;
const VISUALIZER_MIN_HZ: f32 = 35.0;

struct VisualizerProcessor {
    fft: std::sync::Arc<dyn Fft<f32>>,
    fft_buffer: Vec<Complex<f32>>,
    smoothed_bars: Vec<f32>,
    adaptive_level: f32,
    fast_energy: f32,
    slow_energy: f32,
}

impl VisualizerProcessor {
    fn new(bar_count: usize) -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(VISUALIZER_FFT_SIZE);
        Self {
            fft,
            fft_buffer: vec![Complex::zero(); VISUALIZER_FFT_SIZE],
            smoothed_bars: vec![0.0; bar_count.max(1)],
            adaptive_level: 0.08,
            fast_energy: 0.0,
            slow_energy: 0.0,
        }
    }

    fn reset(&mut self) {
        self.smoothed_bars.fill(0.0);
        self.adaptive_level = 0.08;
        self.fast_energy = 0.0;
        self.slow_energy = 0.0;
    }

    fn ensure_bar_count(&mut self, count: usize) {
        let count = count.max(1);
        if self.smoothed_bars.len() != count {
            self.smoothed_bars = vec![0.0; count];
            self.adaptive_level = 0.08;
        }
    }

    fn decay_only(&mut self, out: &mut [f32]) -> i32 {
        self.ensure_bar_count(out.len());
        for value in &mut self.smoothed_bars {
            *value *= 0.93;
        }
        self.fast_energy *= 0.90;
        self.slow_energy *= 0.97;
        for (index, value) in out.iter_mut().enumerate() {
            *value = self.smoothed_bars[index];
        }
        out.len() as i32
    }

    fn compute(&mut self, samples: &[f32], channels: usize, sample_rate: u32, out: &mut [f32], playing: bool) -> i32 {
        if out.is_empty() {
            return 0;
        }

        self.ensure_bar_count(out.len());

        if !playing || samples.is_empty() || channels == 0 || sample_rate == 0 {
            return self.decay_only(out);
        }

        let frame_count = samples.len() / channels;
        if frame_count == 0 {
            return self.decay_only(out);
        }

        let window_frames = VISUALIZER_FFT_SIZE.min(frame_count);
        let start_frame = frame_count.saturating_sub(window_frames);

        self.fft_buffer.fill(Complex::zero());

        let mut rms_acc = 0.0f32;
        for index in 0..window_frames {
            let src_frame = start_frame + index;
            let mut mixed = 0.0f32;
            for ch in 0..channels {
                mixed += samples[src_frame * channels + ch];
            }
            let mono = mixed / channels as f32;
            rms_acc += mono * mono;

            let target = VISUALIZER_FFT_SIZE - window_frames + index;
            self.fft_buffer[target].re = mono * hann_window(index, window_frames);
        }

        self.fft.process(&mut self.fft_buffer);

        let half = VISUALIZER_FFT_SIZE / 2;
        let mut magnitudes = vec![0.0f32; half];
        let norm = window_frames.max(1) as f32;
        for (bin, value) in self.fft_buffer.iter().take(half).enumerate() {
            magnitudes[bin] = (value.re * value.re + value.im * value.im).sqrt() / norm;
        }

        let rms = (rms_acc / window_frames.max(1) as f32).sqrt();
        self.fast_energy = self.fast_energy * 0.50 + rms * 0.50;
        self.slow_energy = self.slow_energy * 0.97 + rms * 0.03;
        let beat = ((self.fast_energy - self.slow_energy) * 11.0).clamp(0.0, 1.0);

        let min_hz = VISUALIZER_MIN_HZ;
        let max_hz = ((sample_rate as f32) * 0.46).max(min_hz + 1.0);
        let bar_count = out.len();
        let mut raw_bars = vec![0.0f32; bar_count];

        for bar in 0..bar_count {
            let t0 = bar as f32 / bar_count as f32;
            let t1 = (bar + 1) as f32 / bar_count as f32;
            let f0 = log_interp(min_hz, max_hz, t0);
            let f1 = log_interp(min_hz, max_hz, t1);

            let b0 = hz_to_bin(f0, sample_rate, VISUALIZER_FFT_SIZE).max(1);
            let b1 = hz_to_bin(f1, sample_rate, VISUALIZER_FFT_SIZE).max(b0 + 1);
            let end = b1.min(magnitudes.len());
            let start = b0.min(end.saturating_sub(1));

            let mut energy = 0.0f32;
            let mut count = 0usize;
            for value in &magnitudes[start..end] {
                energy += *value;
                count += 1;
            }

            let mut raw = if count > 0 { energy / count as f32 } else { 0.0 };
            raw *= 1.0 + beat * 0.55 * (1.0 - t0);
            raw_bars[bar] = raw;
        }

        let frame_peak = raw_bars.iter().copied().fold(0.0f32, f32::max);
        self.adaptive_level = self.adaptive_level * 0.95 + frame_peak.max(0.0001) * 0.05;
        let level = self.adaptive_level.max(0.0001);

        let mut spatial = vec![0.0f32; bar_count];
        for index in 0..bar_count {
            let left = if index > 0 { raw_bars[index - 1] } else { raw_bars[index] };
            let center = raw_bars[index];
            let right = if index + 1 < bar_count {
                raw_bars[index + 1]
            } else {
                raw_bars[index]
            };
            spatial[index] = left * 0.20 + center * 0.60 + right * 0.20;
        }

        for (index, raw) in spatial.iter().enumerate() {
            let mut target = (raw / level).clamp(0.0, 2.0);
            target = target.powf(0.78) * 0.70;
            target = target.clamp(0.0, 1.0);

            let current = self.smoothed_bars[index];
            let alpha = if target > current { 0.34 } else { 0.08 };
            self.smoothed_bars[index] = current + (target - current) * alpha;
            out[index] = self.smoothed_bars[index];
        }

        bar_count as i32
    }
}

#[derive(Clone)]
pub enum AudioSource {
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
    visualizer_ring: VecDeque<f32>,
    visualizer_max_samples: usize,
    max_samples: usize,
    max_queue_seconds: usize,
    playing: bool,
    stream_finished: bool,
    volume: f32,
    lowpass_hz: f32,
    lowpass_alpha: f32,
    lowpass_prev: Vec<f32>,
    emitted_samples: u64,
    source_offset_samples: u64,
    sample_rate: u32,
}

impl SharedPlayback {
    fn new(channels: usize, sample_rate: u32) -> Self {
        let max_samples = sample_rate
            .saturating_mul(channels as u32)
            .saturating_mul(DEFAULT_MAX_QUEUE_SECONDS as u32) as usize;
        let visualizer_max_samples = sample_rate
            .saturating_mul(channels as u32)
            .saturating_mul(DEFAULT_VISUALIZER_SECONDS as u32) as usize;
        Self {
            queue: VecDeque::with_capacity(max_samples),
            visualizer_ring: VecDeque::with_capacity(visualizer_max_samples),
            visualizer_max_samples,
            max_samples,
            max_queue_seconds: DEFAULT_MAX_QUEUE_SECONDS,
            playing: false,
            stream_finished: true,
            volume: 1.0,
            lowpass_hz: 0.0,
            lowpass_alpha: 0.0,
            lowpass_prev: vec![0.0; channels],
            emitted_samples: 0,
            source_offset_samples: 0,
            sample_rate,
        }
    }

    fn clear_audio_state(&mut self) {
        self.queue.clear();
        self.visualizer_ring.clear();
        self.lowpass_prev.fill(0.0);
        self.emitted_samples = 0;
        self.source_offset_samples = 0;
        self.stream_finished = true;
    }

    fn push_visualizer_sample(&mut self, sample: f32) {
        if self.visualizer_max_samples == 0 {
            return;
        }

        if self.visualizer_ring.len() >= self.visualizer_max_samples {
            let _ = self.visualizer_ring.pop_front();
        }

        self.visualizer_ring.push_back(sample);
    }

    fn copy_latest_visualizer_samples(&self, out: &mut [f32]) -> usize {
        if out.is_empty() || self.visualizer_ring.is_empty() {
            return 0;
        }

        let count = out.len().min(self.visualizer_ring.len());
        let skip = self.visualizer_ring.len().saturating_sub(count);

        for (index, sample) in self
            .visualizer_ring
            .iter()
            .skip(skip)
            .take(count)
            .enumerate()
        {
            out[index] = *sample;
        }

        count
    }

    fn recalc_lowpass_alpha(&mut self) {
        if self.lowpass_hz <= 0.0 {
            self.lowpass_alpha = 0.0;
            return;
        }

        let dt = 1.0 / self.sample_rate.max(1) as f32;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * self.lowpass_hz.max(1.0));
        self.lowpass_alpha = dt / (rc + dt);
    }

    fn set_max_queue_seconds(&mut self, channels: usize, seconds: usize) {
        let bounded = seconds.clamp(MIN_MAX_QUEUE_SECONDS, MAX_MAX_QUEUE_SECONDS);
        self.max_queue_seconds = bounded;
        self.max_samples = self
            .sample_rate
            .saturating_mul(channels as u32)
            .saturating_mul(bounded as u32) as usize;

        while self.queue.len() > self.max_samples {
            let _ = self.queue.pop_front();
        }

        if self.queue.capacity() < self.max_samples {
            self.queue
                .reserve(self.max_samples.saturating_sub(self.queue.capacity()));
        }
    }

    fn push_samples_bounded(&mut self, samples: &[f32]) -> usize {
        let available = self.max_samples.saturating_sub(self.queue.len());
        let push_count = samples.len().min(available);
        self.queue.extend(samples.iter().take(push_count).copied());
        push_count
    }

    fn next_sample(&mut self, channel: usize) -> f32 {
        if !self.playing {
            return 0.0;
        }

        let raw = if let Some(sample) = self.queue.pop_front() {
            self.emitted_samples = self.emitted_samples.saturating_add(1);
            sample
        } else {
            if self.stream_finished {
                self.playing = false;
            }
            return 0.0;
        };

        let mut sample = raw * self.volume;

        if self.lowpass_alpha > 0.0 {
            let alpha = self.lowpass_alpha;
            let prev = self.lowpass_prev[channel];
            let filtered = prev + alpha * (sample - prev);
            self.lowpass_prev[channel] = filtered;
            sample = filtered;
        }

        let sample = sample.clamp(-1.0, 1.0);
        self.push_visualizer_sample(sample);
        sample
    }
}

pub struct AudioEngine {
    shared: Arc<Mutex<SharedPlayback>>,
    stream_started: bool,
    source: Option<AudioSource>,
    decode_thread: Option<JoinHandle<()>>,
    decode_stop: Arc<AtomicBool>,
    out_channels: usize,
    out_sample_rate: u32,
    visualizer_processor: VisualizerProcessor,
    source_duration_millis: i32,
    decode_start_millis: i32,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No default output device available".to_string())?;

        let config = device
            .default_output_config()
            .map_err(|e| format!("Failed to read default output config: {e}"))?;

        let out_channels = config.channels() as usize;
        let out_sample_rate = config.sample_rate();

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
            visualizer_processor: VisualizerProcessor::new(DEFAULT_VISUALIZER_BAR_COUNT),
            source_duration_millis: -1,
            decode_start_millis: 0,
        })
    }

    pub fn ensure_stream(&mut self) -> Result<(), String> {
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
            sample_rate: (self.out_sample_rate),
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

    pub fn set_source(&mut self, source: AudioSource) {
        self.source_duration_millis = estimate_duration_millis(&source, self.out_channels as u32);
        self.decode_start_millis = 0;
        self.source = Some(source);
        self.stop_decode_thread();
        if let Ok(mut shared) = self.shared.lock() {
            shared.clear_audio_state();
            shared.stream_finished = false;
        }
        self.visualizer_processor.reset();
    }

    pub fn set_playing(&mut self, playing: bool) {
        if let Ok(mut shared) = self.shared.lock() {
            if playing {
                shared.stream_finished = false;
            }
            shared.playing = playing;
        }
    }

    pub fn stop(&mut self) {
        self.set_playing(false);
        self.stop_decode_thread();
        if let Ok(mut shared) = self.shared.lock() {
            shared.clear_audio_state();
        }
        self.visualizer_processor.reset();
    }

    pub fn set_volume(&mut self, volume: f32) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.volume = volume.clamp(0.0, 4.0);
        }
    }

    pub fn set_lowpass_hz(&mut self, hz: f32) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.lowpass_hz = hz.max(0.0);
            shared.recalc_lowpass_alpha();
            shared.lowpass_prev.fill(0.0);
        }
    }

    pub fn set_max_queue_seconds(&mut self, seconds: usize) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.set_max_queue_seconds(self.out_channels, seconds);
        }
    }

    pub fn seek(&mut self, millis: i32) {
        let mut target = millis.max(0);
        if self.source_duration_millis > 0 {
            target = target.min(self.source_duration_millis);
        }

        let should_resume = self.is_playing() == 1;

        self.stop_decode_thread();
        self.decode_start_millis = target;

        let target_samples = ((target as u64)
            .saturating_mul(self.out_sample_rate as u64)
            .saturating_mul(self.out_channels as u64)
            / 1000) as u64;

        if let Ok(mut shared) = self.shared.lock() {
            shared.queue.clear();
            shared.visualizer_ring.clear();
            shared.lowpass_prev.fill(0.0);
            shared.emitted_samples = 0;
            shared.source_offset_samples = target_samples;
            shared.stream_finished = false;
            shared.playing = false;
        }
        self.visualizer_processor.reset();

        if should_resume {
            let _ = self.start_decode_thread_if_needed();
            self.set_playing(true);
        }
    }

    pub fn position_millis(&self) -> i32 {
        match self.shared.lock() {
            Ok(shared) => {
                if shared.sample_rate == 0 || self.out_channels == 0 {
                    return 0;
                }

                let total_output_samples =
                    shared.source_offset_samples.saturating_add(shared.emitted_samples);

                (total_output_samples as f64
                    / (shared.sample_rate as f64 * self.out_channels as f64)
                    * 1000.0) as i32
            }
            Err(_) => -1,
        }
    }

    pub fn duration_millis(&self) -> i32 {
        self.source_duration_millis
    }

    pub fn max_queue_seconds(&self) -> i32 {
        match self.shared.lock() {
            Ok(shared) => shared.max_queue_seconds as i32,
            Err(_) => -1,
        }
    }

    pub fn buffered_samples(&self) -> i32 {
        match self.shared.lock() {
            Ok(shared) => shared.queue.len() as i32,
            Err(_) => -1,
        }
    }

    pub fn buffered_millis(&self) -> i32 {
        match self.shared.lock() {
            Ok(shared) => {
                if shared.sample_rate == 0 {
                    return 0;
                }
                ((shared.queue.len() as f64 / self.out_channels.max(1) as f64)
                    / shared.sample_rate as f64
                    * 1000.0) as i32
            }
            Err(_) => -1,
        }
    }

    pub fn visualizer_available_samples(&self) -> i32 {
        match self.shared.lock() {
            Ok(shared) => shared.visualizer_ring.len() as i32,
            Err(_) => -1,
        }
    }

    pub fn visualizer_sample_rate(&self) -> i32 {
        self.out_sample_rate as i32
    }

    pub fn visualizer_channels(&self) -> i32 {
        self.out_channels as i32
    }

    pub fn copy_visualizer_samples(&self, out: &mut [f32]) -> i32 {
        match self.shared.lock() {
            Ok(shared) => shared.copy_latest_visualizer_samples(out) as i32,
            Err(_) => -1,
        }
    }

    pub fn copy_visualizer_spectrum(&mut self, out: &mut [f32]) -> i32 {
        let (snapshot, playing) = match self.shared.lock() {
            Ok(shared) => (shared.visualizer_ring.iter().copied().collect::<Vec<f32>>(), shared.playing),
            Err(_) => return -1,
        };

        self.visualizer_processor
            .compute(&snapshot, self.out_channels, self.out_sample_rate, out, playing)
    }

    pub fn is_playing(&self) -> i32 {
        match self.shared.lock() {
            Ok(shared) => i32::from(shared.playing),
            Err(_) => -1,
        }
    }

    pub fn is_source_loaded(&self) -> i32 {
        i32::from(self.source.is_some())
    }

    pub fn start_decode_thread_if_needed(&mut self) -> Result<(), String> {
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
        let start_millis = self.decode_start_millis;

        if let Ok(mut state) = self.shared.lock() {
            state.stream_finished = false;
        }

        let handle = thread::spawn(move || {
            if let Err(err) = decode_and_feed(
                source,
                stop_flag,
                shared,
                out_channels,
                out_sample_rate,
                start_millis,
            ) {
                error!("Decode thread ended with error: {err}");
            }
        });

        self.decode_thread = Some(handle);
        Ok(())
    }

    pub fn stop_decode_thread(&mut self) {
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

type BoxedMediaSource = Box<dyn symphonia::core::io::MediaSource>;

fn write_bytes_to_temp_file(bytes: &[u8], tag: &str) -> Result<File, String> {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    path.push(format!("audiopc_{tag}_{pid}_{nanos}.bin"));

    std::fs::write(&path, bytes)
        .map_err(|e| format!("Failed to write temporary media file '{}': {e}", path.display()))?;

    File::open(&path)
        .map_err(|e| format!("Failed to open temporary media file '{}': {e}", path.display()))
}

fn media_source_from_owned(source: AudioSource) -> Result<BoxedMediaSource, String> {
    match source {
        AudioSource::Path(path) => {
            let file = File::open(&path)
                .map_err(|e| format!("Failed to open file source '{path}': {e}"))?;
            let source: BoxedMediaSource = Box::new(file);
            Ok(source)
        }
        AudioSource::Url(url) => {
            let response = reqwest::blocking::get(&url)
                .and_then(|res| res.error_for_status())
                .map_err(|e| format!("Failed to fetch URL source '{url}': {e}"))?;
            let bytes = response
                .bytes()
                .map_err(|e| format!("Failed to read response body: {e}"))?;
            let file = write_bytes_to_temp_file(&bytes, "url")?;
            let source: BoxedMediaSource = Box::new(file);
            Ok(source)
        }
        AudioSource::Memory(data) => {
            let file = write_bytes_to_temp_file(&data, "memory")?;
            let source: BoxedMediaSource = Box::new(file);
            Ok(source)
        }
    }
}

fn media_source_from_ref(source: &AudioSource) -> Result<BoxedMediaSource, String> {
    match source {
        AudioSource::Path(path) => {
            let file = File::open(path)
                .map_err(|e| format!("Failed to open source for duration: {e}"))?;
            let source: BoxedMediaSource = Box::new(file);
            Ok(source)
        }
        AudioSource::Url(url) => {
            let response = reqwest::blocking::get(url)
                .and_then(|res| res.error_for_status())
                .map_err(|e| format!("Failed to fetch URL for duration: {e}"))?;
            let bytes = response
                .bytes()
                .map_err(|e| format!("Failed to read URL body for duration: {e}"))?;
            let file = write_bytes_to_temp_file(&bytes, "duration_url")?;
            let source: BoxedMediaSource = Box::new(file);
            Ok(source)
        }
        AudioSource::Memory(data) => {
            let file = write_bytes_to_temp_file(data, "duration_memory")?;
            let source: BoxedMediaSource = Box::new(file);
            Ok(source)
        }
    }
}

fn decode_and_feed(
    source: AudioSource,
    stop_flag: Arc<AtomicBool>,
    shared: Arc<Mutex<SharedPlayback>>,
    out_channels: usize,
    out_sample_rate: u32,
    start_millis: i32,
) -> Result<(), String> {
    let media_source = media_source_from_owned(source)?;

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
    let mut skip_output_samples = ((start_millis.max(0) as u64)
        .saturating_mul(out_sample_rate as u64)
        .saturating_mul(out_channels as u64)
        / 1000) as usize;

    let decode_result = loop {
        if stop_flag.load(Ordering::SeqCst) {
            break Ok(());
        }

        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::ResetRequired) => {
                break Err("Decoder reset required and not supported".to_string())
            }
            Err(SymphoniaError::IoError(_)) => break Ok(()),
            Err(err) => break Err(format!("Failed to read next packet: {err}")),
        };

        let decoded = match decoder.decode(&packet) {
            Ok(buf) => buf,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(_)) => break Ok(()),
            Err(err) => break Err(format!("Failed to decode packet: {err}")),
        };

        let (src_channels, src_rate, interleaved) = decoded_to_interleaved_f32(decoded);
        if interleaved.is_empty() || src_channels == 0 || src_rate == 0 {
            continue;
        }

        if stop_flag.load(Ordering::SeqCst) {
            break Ok(());
        }

        let out = convert_to_output(
            &interleaved,
            src_channels,
            src_rate,
            out_channels,
            out_sample_rate,
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

        let mut offset = 0;
        let out_slice = &out[start..];
        while offset < out_slice.len() {
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }

            let pushed = if let Ok(mut s) = shared.lock() {
                s.push_samples_bounded(&out_slice[offset..])
            } else {
                0
            };

            if pushed == 0 {
                thread::sleep(Duration::from_millis(DECODE_BACKPRESSURE_SLEEP_MS));
                continue;
            }

            offset += pushed;
        }
    };

    if let Ok(mut state) = shared.lock() {
        state.stream_finished = true;
    }

    decode_result
}

fn estimate_duration_millis(source: &AudioSource, _out_channels: u32) -> i32 {
    fn from_codec_params(params: &symphonia::core::codecs::CodecParameters) -> i32 {
        if let (Some(n_frames), Some(sample_rate)) = (params.n_frames, params.sample_rate) {
            return ((n_frames as f64 / sample_rate as f64) * 1000.0) as i32;
        }

        -1
    }

    let media_source = match media_source_from_ref(source) {
        Ok(src) => src,
        Err(err) => {
            error!("{err}");
            return -1;
        }
    };

    let mss = MediaSourceStream::new(media_source, MediaSourceStreamOptions::default());
    let hint = Hint::new();
    let probed = match symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    ) {
        Ok(p) => p,
        Err(err) => {
            error!("Failed to probe source for duration: {err}");
            return -1;
        }
    };

    let format = probed.format;
    let track = match format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
    {
        Some(t) => t,
        None => return -1,
    };

    from_codec_params(&track.codec_params)
}

fn decoded_to_interleaved_f32(decoded: AudioBufferRef<'_>) -> (usize, u32, Vec<f32>) {
    let spec = *decoded.spec();
    let channels = spec.channels.count();
    let sample_rate = spec.rate;
    let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
    sample_buf.copy_interleaved_ref(decoded);
    (channels, sample_rate, sample_buf.samples().to_vec())
}

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
    if total_frames < 2 {
        state.carry = frames;
        return Vec::new();
    }

    let step = src_rate as f64 / out_rate as f64;
    let mut pos = state.pos;

    let estimated_frames = (((total_frames as f64 - pos - 1.0) / step)
        .max(0.0)
        .ceil() as usize)
        .saturating_add(1);
    let mut out = Vec::with_capacity(estimated_frames.saturating_mul(out_channels));

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
    state.carry.clear();
    state
        .carry
        .extend_from_slice(&frames[keep_base..keep_base + src_channels]);
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

pub fn default_output_sample_rate() -> i32 {
    let host = cpal::default_host();
    let Some(device) = host.default_output_device() else {
        return -1;
    };

    match device.default_output_config() {
        Ok(config) => config.sample_rate() as i32,
        Err(_) => -1,
    }
}

pub fn default_output_channels() -> i32 {
    let host = cpal::default_host();
    let Some(device) = host.default_output_device() else {
        return -1;
    };

    match device.default_output_config() {
        Ok(config) => i32::from(config.channels()),
        Err(_) => -1,
    }
}

pub fn output_device_count() -> i32 {
    let host = cpal::default_host();
    match host.output_devices() {
        Ok(devices) => devices.count() as i32,
        Err(_) => -1,
    }
}

fn hann_window(index: usize, len: usize) -> f32 {
    if len <= 1 {
        return 1.0;
    }
    let x = index as f32 / (len - 1) as f32;
    (0.5 - 0.5 * (2.0 * std::f32::consts::PI * x).cos()).clamp(0.0, 1.0)
}

fn log_interp(min: f32, max: f32, t: f32) -> f32 {
    min * (max / min).powf(t.clamp(0.0, 1.0))
}

fn hz_to_bin(freq: f32, sample_rate: u32, fft_size: usize) -> usize {
    let nyquist = sample_rate as f32 / 2.0;
    let clamped = freq.clamp(0.0, nyquist);
    ((clamped / sample_rate as f32) * fft_size as f32) as usize
}
