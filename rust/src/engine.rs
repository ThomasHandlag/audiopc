/// Audio playback engine.
///
/// `AudioEngine` is the central coordinator between:
///
/// * The **device** layer ([`crate::device::DeviceManager`]) — enumerates and
///   selects the cpal output device.
/// * The **decode thread** — pulls packets from a [`crate::source::AudioSource`],
///   resamples them to the device rate, and pushes interleaved `f32` samples
///   into the shared queue.
/// * The **cpal callback** — drains the queue on the audio thread, applies
///   per-channel DSP effects, and writes to the hardware buffer.
/// * The **event channel** — broadcasts [`crate::events::AudioEvent`] to any
///   number of subscribers (UI, logging, test harness …).
///
/// # Design contracts
///
/// * The cpal callback is **never blocked**.  All heavy work (disk I/O,
///   network, decoding) happens on a separate thread feeding a `VecDeque<f32>`
///   with backpressure via [`crate::enums::DECODE_BACKPRESSURE_SLEEP_MS`].
/// * `AudioEngine` is `Send + Sync` (the cpal `Stream` is kept alive but not
///   moved after construction).
/// * Errors surface through `Result<_, String>` (legacy FFI compat) and via
///   `AudioEvent::Error`.

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{BufferSize, SampleFormat, Stream, StreamConfig, StreamError};

use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use tempfile::tempfile;

use crate::debug;
use crate::device::DeviceManager;
use crate::effects::{
    band_pass_filter, high_shelf_filter, highpass_filter, low_shelf_filter,
    lowpass_filter, notch_filter, peak_filter,
};
use crate::enums::{
    DECODE_BACKPRESSURE_SLEEP_MS, DEFAULT_VISUALIZER_BAR_COUNT, MAX_RATE, MIN_RATE,
};
use crate::error::AudioError;
use crate::events::{event_channel};
use crate::http_stream::HttpStream;
use crate::player_state::{PlaybackStatus, PlayerState, ResampleState, SharedPlayback};
use crate::processor::VisualizerProcessor;
use crate::source::AudioSource;
use crate::{error, info, warn};

// ── Internal type aliases ─────────────────────────────────────────────────────

type BoxedMediaSource = Box<dyn symphonia::core::io::MediaSource>;

// ── AudioEngine ───────────────────────────────────────────────────────────────

/// Main audio engine struct.
///
/// Owns all engine state.  Constructed once; most control methods take
/// `&mut self` because they potentially restart threads.
pub struct AudioEngine {
    // ── Shared audio-callback / decode-thread state ────────────────────────
    shared: Arc<Mutex<SharedPlayback>>,

    // ── cpal stream (kept alive for its lifetime) ──────────────────────────
    audio_stream:  Option<Stream>,
    stream_started: bool,

    // ── Device info ────────────────────────────────────────────────────────
    out_channels:    usize,
    out_sample_rate: u32,
    /// Optional preferred device name; `None` = system default.
    preferred_device: Option<String>,

    // ── Source ────────────────────────────────────────────────────────────
    source: Option<AudioSource>,

    // ── Decode thread ─────────────────────────────────────────────────────
    decode_thread: Option<JoinHandle<()>>,
    decode_stop:   Arc<AtomicBool>,

    // ── Seek / timing ──────────────────────────────────────────────────────
    source_duration_millis: i32,
    decode_start_millis:    i32,

    // ── Visualizer ────────────────────────────────────────────────────────
    visualizer_processor: VisualizerProcessor,

    // ── Device watcher ─────────────────────────────────────────────────────
    /// Set to `true` to stop the device watcher thread.
    device_watcher_stop: Arc<AtomicBool>,
}

impl AudioEngine {
    /// Create a new engine using the platform default audio host and output
    /// device.
    pub fn new() -> Result<Self, String> {
        Self::with_device(None)
    }

    /// Create a new engine, preferring `device_name` for output.
    ///
    /// If `device_name` is `None`, or the named device is not found, the
    /// system default is used.
    pub fn with_device(device_name: Option<String>) -> Result<Self, String> {
        let manager = DeviceManager::new();
        let device  = manager
            .resolve_output(device_name.as_deref())
            .or_else(|_| manager.resolve_output(None))
            .map_err(|e| e.to_string())?;

        let config = device
            .default_output_config()
            .map_err(|e| AudioError::from(e).to_string())?;

        let out_channels    = config.channels() as usize;
        // cpal 0.17: SupportedStreamConfig::sample_rate() returns u32.
        let out_sample_rate: u32 = config.sample_rate();

        let (event_tx, _event_rx) = event_channel();
        // Note: the initial receiver is not used by the engine. Consumers can
        // call `event_sender().subscribe()` to get their own receiver stream.

        // Start the device watcher in the background.
        let device_watcher_stop =
            crate::device::start_device_watcher(event_tx.clone());

        Ok(Self {
            shared:                  Arc::new(Mutex::new(SharedPlayback::new(
                out_channels,
                out_sample_rate,
            ))),
            audio_stream:            None,
            stream_started:          false,
            out_channels,
            out_sample_rate,
            preferred_device:        device_name,
            source:                  None,
            decode_thread:           None,
            decode_stop:             Arc::new(AtomicBool::new(false)),
            source_duration_millis:  -1,
            decode_start_millis:     0,
            visualizer_processor:    VisualizerProcessor::new(DEFAULT_VISUALIZER_BAR_COUNT),
            device_watcher_stop,
        })
    }

    // ── Stream lifecycle ──────────────────────────────────────────────────

    /// Initialise the cpal output stream if not already done.  Called lazily
    /// at first `play()` to avoid issues where the device isn't ready at
    /// startup (common on Android).
    pub fn ensure_stream(&mut self) -> Result<(), String> {
        if self.stream_started {
            return Ok(());
        }

        info!("Init Stream");

        let manager = DeviceManager::new();
        let device  = manager
            .resolve_output(self.preferred_device.as_deref())
            .or_else(|_| manager.resolve_output(None))
            .map_err(|e| e.to_string())?;

        let output_config = device
            .default_output_config()
            .map_err(|e| AudioError::from(e).to_string())?;

        let sample_format = output_config.sample_format();
        // cpal 0.17: SupportedStreamConfig::sample_rate() returns u32 directly.
        let stream_config = StreamConfig {
            channels:    output_config.channels(),
            sample_rate: output_config.sample_rate(),
            buffer_size: BufferSize::Default,
        };

        let shared   = Arc::clone(&self.shared);
        let channels = self.out_channels;

        let err_fn = {
            move |err: StreamError| {
                error!("Stream Error: {err}");
                thread::spawn(|| {
                    // Signal the FFI layer to rebuild the stream.
                    crate::ffi::revise_stream();
                });
            }
        };

        let stream = match sample_format {
            SampleFormat::F32 =>
                device.build_output_stream(
                    &stream_config,
                    {
                        let shared = Arc::clone(&shared);
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            write_output_f32(data, channels, &shared);
                        }
                    },
                    err_fn,
                    None,
                ).map_err(|e| AudioError::from(e).to_string())?,

            SampleFormat::I16 =>
                device.build_output_stream(
                    &stream_config,
                    {
                        let shared = Arc::clone(&shared);
                        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                            write_output_i16(data, channels, &shared);
                        }
                    },
                    err_fn,
                    None,
                ).map_err(|e| AudioError::from(e).to_string())?,

            SampleFormat::U16 =>
                device.build_output_stream(
                    &stream_config,
                    {
                        let shared = Arc::clone(&shared);
                        move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                            write_output_u16(data, channels, &shared);
                        }
                    },
                    err_fn,
                    None,
                ).map_err(|e| AudioError::from(e).to_string())?,

            _ => return Err("Unsupported sample format".to_string()),
        };

        stream
            .play()
            .map_err(|e| AudioError::from(e).to_string())?;

        self.audio_stream  = Some(stream);
        self.stream_started = true;
        Ok(())
    }

    /// Stop, rebuild and resume the stream.  Called when cpal reports an
    /// unrecoverable stream error (device disconnected, etc.).
    pub fn reset_stream(&mut self) -> Result<(), String> {
        self.stream_started = false;
        self.stop_decode_thread();
        if let Ok(mut s) = self.shared.lock() {
            s.stream_finished = false;
            s.playing         = false;
            s.queue.clear();
            s.visualizer_ring.clear();
        }
        self.ensure_stream()?;
        self.start_decode_thread_if_needed()?;
        self.set_playing(true);
        Ok(())
    }

    // ── Source management ─────────────────────────────────────────────────

    /// Load a new audio source, stopping any currently playing source.
    pub fn set_source(&mut self, source: AudioSource) {
        info!("Set source: {}", source.description());
        self.source_duration_millis =
            estimate_duration_millis(&source, self.out_channels as u32);
        self.decode_start_millis = 0;
        self.source              = Some(source.clone());
        self.stop_decode_thread();
        if let Ok(mut s) = self.shared.lock() {
            s.clear_audio_state();
            s.stream_finished = false;
            s.status          = PlaybackStatus::Idle;
        }
        self.visualizer_processor.reset();
    }

    // ── Playback control ──────────────────────────────────────────────────

    pub fn set_playing(&mut self, playing: bool) {
        if let Ok(mut s) = self.shared.lock() {
            if playing {
                s.stream_finished = false;
                s.status          = PlaybackStatus::Playing;
            } else {
                s.status = PlaybackStatus::Paused;
            }
            s.playing = playing;
        }
    }

    pub fn stop(&mut self) {
        self.set_playing(false);
        self.stop_decode_thread();
        if let Ok(mut s) = self.shared.lock() {
            s.clear_audio_state();
            s.status = PlaybackStatus::Idle;
        }
        self.visualizer_processor.reset();
    }

    pub fn set_volume(&mut self, volume: f32) {
        if let Ok(mut s) = self.shared.lock() {
            s.volume = volume.clamp(0.0, 4.0);
        }
    }

    pub fn set_max_queue_seconds(&mut self, seconds: usize) {
        if let Ok(mut s) = self.shared.lock() {
            s.set_max_queue_seconds(self.out_channels, seconds);
        }
    }

    // ── Seek ──────────────────────────────────────────────────────────────

    pub fn seek(&mut self, millis: i32) {
        let mut target = millis.max(0);
        if self.source_duration_millis > 0 {
            target = target.min(self.source_duration_millis);
        }

        if !matches!(self.source, Some(_)) {
            warn!("Seek called with no source");
            return;
        }

        let was_playing = self.is_playing() == 1;

        self.stop_decode_thread();
        self.decode_start_millis = target;

        let target_samples = ((target as u64)
            .saturating_mul(self.out_sample_rate as u64)
            .saturating_mul(self.out_channels as u64)
            / 1000) as u64;

        if let Ok(mut s) = self.shared.lock() {
            s.queue.clear();
            s.visualizer_ring.clear();
            s.emitted_samples         = 0;
            s.source_position_samples = target_samples as f64;
            s.stream_finished         = false;
            s.playing                 = false;
        }
        self.visualizer_processor.reset();

        let can_play = self.source.is_some() && millis < self.source_duration_millis;

        if was_playing || can_play {
            let _ = self.start_decode_thread_if_needed();
            self.set_playing(true);
        }
    }

    // ── Rate / pitch ──────────────────────────────────────────────────────

    pub fn set_rate(&mut self, rate: f32) {
        let rate = rate.clamp(MIN_RATE, MAX_RATE);
        let was_playing = self.is_playing() == 1;
        let current_pos = self.position_millis().max(0);

        if let Ok(mut s) = self.shared.lock() {
            s.playback_rate = rate;
        }

        self.stop_decode_thread();
        self.decode_start_millis = current_pos;

        let target_samples = ((current_pos as u64)
            .saturating_mul(self.out_sample_rate as u64)
            .saturating_mul(self.out_channels as u64)
            / 1000) as f64;

        if let Ok(mut s) = self.shared.lock() {
            s.queue.clear();
            s.visualizer_ring.clear();
            s.emitted_samples         = 0;
            s.source_position_samples = target_samples;
            s.stream_finished         = false;
            s.playing                 = false;
        }
        self.visualizer_processor.reset();

        if self.source.is_some() {
            let _ = self.start_decode_thread_if_needed();
            if was_playing {
                self.set_playing(true);
            }
        }
    }

    pub fn rate(&self) -> f32 {
        self.shared.lock().map(|s| s.playback_rate).unwrap_or(1.0)
    }

    // ── Position / duration ───────────────────────────────────────────────

    pub fn position_millis(&self) -> i32 {
        self.shared
            .lock()
            .map(|s| s.position_millis(self.out_channels))
            .unwrap_or(-1)
    }

    pub fn duration_millis(&self) -> i32 {
        self.source_duration_millis
    }

    pub fn max_queue_seconds(&self) -> i32 {
        self.shared.lock().map(|s| s.max_queue_seconds as i32).unwrap_or(-1)
    }

    pub fn buffered_samples(&self) -> i32 {
        self.shared.lock().map(|s| s.queue.len() as i32).unwrap_or(-1)
    }

    pub fn buffered_millis(&self) -> i32 {
        self.shared.lock()
            .map(|s| {
                if s.sample_rate == 0 { return 0; }
                ((s.queue.len() as f64 / self.out_channels.max(1) as f64)
                    / s.sample_rate as f64
                    * 1000.0) as i32
            })
            .unwrap_or(-1)
    }

    // ── Playback state ────────────────────────────────────────────────────

    pub fn is_playing(&self) -> i32 {
        self.shared.lock().map(|s| i32::from(s.playing)).unwrap_or(-1)
    }

    pub fn get_state(&self) -> PlayerState {
        if self.source.is_none() {
            return PlayerState::Idle;
        }
        self.shared
            .lock()
            .map(|s| PlayerState::from(&s.status))
            .unwrap_or(PlayerState::Idle)
    }

    // ── DSP / filters ─────────────────────────────────────────────────────

    /// Replace the effect chain of every output channel with a fresh, empty
    /// chain.
    pub fn clear_filters(&mut self) {
        if let Ok(mut s) = self.shared.lock() {
            for e in &mut s.effects {
                e.clear();
            }
        }
    }

    /// Validate that filter parameters are sensible before creating
    /// coefficients.  Returns `0` on success, `-1` on fatal errors,
    /// and logs a warning (but still returns `0`) if the filter should be
    /// disabled by the caller.
    pub fn filter_check(&self, cutoff_hz: f32, q: f32) -> i8 {
        let fs = match self.shared.lock() {
            Ok(g) => g.sample_rate as f32,
            Err(_) => { error!("Failed to acquire lock for filter check"); return -1; }
        };
        if q <= 0.0 {
            error!("Invalid filter Q: {q}. Must be > 0.");
            return -1;
        }
        if cutoff_hz <= 0.0 {
            warn!("Disabling filter because cutoff_hz is {cutoff_hz}.");
            return 0;
        }
        if cutoff_hz >= fs / 2.0 {
            error!("Invalid cutoff frequency: {cutoff_hz} Hz. Must be < Nyquist ({} Hz).", fs / 2.0);
            return -1;
        }
        info!("Filter params: cutoff={cutoff_hz} Hz, Q={q}");
        0
    }

    // ── Generic helper: replace the named filter in every channel's chain ──

    /// Remove any existing filter with `old_name` from every channel's chain,
    /// then push `new_filter` (if `Some`) to the end.
    ///
    /// This provides an idempotent "set" semantic: calling with the same
    /// parameters twice does not double-apply the effect.
    fn set_filter_named<F>(&mut self, old_name: &'static str, make: F)
    where
        F: Fn(u32) -> Option<crate::effects::BiquadFilter>,
    {
        if let Ok(mut s) = self.shared.lock() {
            let sample_rate = s.sample_rate;
            for effect in &mut s.effects {
                effect.remove_named(old_name);
                if let Some(f) = make(sample_rate) {
                    effect.push(f);
                }
            }
        }
    }

    pub fn set_peak_filter(&mut self, cutoff_hz: f32, gain_db: f32, q: f32) {
        if self.filter_check(cutoff_hz, q) != 0 { return; }
        self.set_filter_named("PeakEQ", move |sr| peak_filter(sr, cutoff_hz, gain_db, q));
    }

    pub fn set_low_shelf_filter(&mut self, cutoff_hz: f32, gain_db: f32, q: f32) {
        if self.filter_check(cutoff_hz, q) != 0 { return; }
        self.set_filter_named("LowShelf", move |sr| low_shelf_filter(sr, cutoff_hz, gain_db, q));
    }

    pub fn set_high_shelf_filter(&mut self, cutoff_hz: f32, gain_db: f32, q: f32) {
        if self.filter_check(cutoff_hz, q) != 0 { return; }
        self.set_filter_named("HighShelf", move |sr| high_shelf_filter(sr, cutoff_hz, gain_db, q));
    }

    pub fn set_band_pass_filter(&mut self, center_hz: f32, q: f32) {
        if self.filter_check(center_hz, q) != 0 { return; }
        self.set_filter_named("BandPass", move |sr| band_pass_filter(sr, center_hz, q));
    }

    pub fn set_lowpass_filter(&mut self, cutoff_hz: f32, q: f32) {
        if self.filter_check(cutoff_hz, q) != 0 { return; }
        self.set_filter_named("LowPass", move |sr| lowpass_filter(sr, cutoff_hz, q));
    }

    pub fn set_high_pass_filter(&mut self, cutoff_hz: f32, q: f32) {
        if self.filter_check(cutoff_hz, q) != 0 { return; }
        self.set_filter_named("HighPass", move |sr| highpass_filter(sr, cutoff_hz, q));
    }

    pub fn set_notch_filter(&mut self, cutoff_hz: f32, q: f32) {
        if self.filter_check(cutoff_hz, q) != 0 { return; }
        self.set_filter_named("Notch", move |sr| notch_filter(sr, cutoff_hz, q));
    }

    // ── Visualizer ────────────────────────────────────────────────────────

    pub fn visualizer_available_samples(&self) -> i32 {
        self.shared.lock().map(|s| s.visualizer_ring.len() as i32).unwrap_or(-1)
    }

    pub fn visualizer_sample_rate(&self) -> i32 { self.out_sample_rate as i32 }
    pub fn visualizer_channels(&self)    -> i32 { self.out_channels as i32 }

    pub fn copy_visualizer_samples(&self, out: &mut [f32]) -> i32 {
        self.shared
            .lock()
            .map(|s| s.copy_latest_visualizer_samples(out) as i32)
            .unwrap_or(-1)
    }

    pub fn copy_visualizer_spectrum(&mut self, out: &mut [f32]) -> i32 {
        let (snapshot, playing) = match self.shared.lock() {
            Ok(s) => (
                s.visualizer_ring.iter().copied().collect::<Vec<f32>>(),
                s.playing,
            ),
            Err(_) => return -1,
        };
        self.visualizer_processor
            .compute(&snapshot, self.out_channels, self.out_sample_rate, out, playing)
    }

    // ── Metadata / thumbnail ──────────────────────────────────────────────

    /// Extract tags and codec info from a media file or URL as a JSON string.
    pub fn get_metadata(&self, path: &str) -> Result<String, String> {
        let media: BoxedMediaSource = open_media_source(path)?;
        let mss  = MediaSourceStream::new(media, MediaSourceStreamOptions::default());
        let mut probed = symphonia::default::get_probe()
            .format(&Hint::new(), mss, &FormatOptions::default(), &MetadataOptions::default())
            .map_err(|_| "Failed to probe format".to_string())?;

        let mut metadata = std::collections::HashMap::<String, String>::new();

        probed.metadata.get().iter().for_each(|cm| {
            if let Some(om) = cm.current() {
                for tag in om.tags() {
                    metadata.insert(tag.key.to_lowercase(), tag.value.to_string());
                }
            }
        });

        let track = probed
            .format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or("No decodable audio track found")?;

        let cp = &track.codec_params;
        if let Some(ch) = cp.channels         { metadata.insert("channels".into(),     ch.count().to_string()); }
        if let Some(sr) = cp.sample_rate      { metadata.insert("sample_rate".into(),  sr.to_string()); }
        if let Some(nf) = cp.n_frames {
            metadata.insert("frame_count".into(), nf.to_string());
            if let Some(sr) = cp.sample_rate {
                let dur = nf as f64 / sr as f64;
                metadata.insert("duration_seconds".into(), dur.to_string());
            }
        }
        metadata.insert("codec".into(), cp.codec.to_string());

        serde_json::to_string(&metadata).map_err(|_| "Failed to serialize metadata".into())
    }

    /// Extract the first embedded image from a media file or URL.
    ///
    /// Returns an empty vec if none is present.
    pub fn get_thumbnail(&self, path: &str) -> Result<Vec<u8>, String> {
        let media: BoxedMediaSource = open_media_source(path)?;
        let mss   = MediaSourceStream::new(media, MediaSourceStreamOptions::default());
        let mut probed = symphonia::default::get_probe()
            .format(&Hint::new(), mss, &FormatOptions::default(), &MetadataOptions::default())
            .map_err(|e| format!("Failed to probe source for thumbnail: {e}"))?;

        for cm in probed.metadata.get().iter() {
            if let Some(om) = cm.current() {
                for v in om.visuals() {
                    if v.media_type == "image/jpeg" || v.media_type == "image/png" {
                        return Ok(v.data.to_vec());
                    }
                }
            }
        }
        Ok(Vec::new())
    }

    // ── Decode thread management ──────────────────────────────────────────

    /// Start the decode thread unless one is already running.
    pub fn start_decode_thread_if_needed(&mut self) -> Result<(), String> {
        if self.decode_thread.is_some() {
            return Ok(());
        }

        let source = self
            .source
            .clone()
            .ok_or_else(|| "No source loaded. Call set_source first.".to_string())?;

        self.decode_stop.store(false, Ordering::SeqCst);
        let stop_flag       = Arc::clone(&self.decode_stop);
        let shared          = Arc::clone(&self.shared);
        let out_channels    = self.out_channels;
        let out_sample_rate = self.out_sample_rate;
        let start_millis    = self.decode_start_millis;

        if let Ok(mut s) = self.shared.lock() {
            s.stream_finished = false;
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

    /// Signal the decode thread to stop and block until it exits.
    pub fn stop_decode_thread(&mut self) {
        self.decode_stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.decode_thread.take() {
            let _ = handle.join();
        }
        self.decode_stop.store(false, Ordering::SeqCst);
    }

    // ── Device info forwarding ────────────────────────────────────────────

    pub fn default_output_sample_rate() -> i32 {
        use cpal::traits::HostTrait;
        cpal::default_host()
            .default_output_device()
            .and_then(|d: cpal::Device| d.default_output_config().ok())
            .map(|c: cpal::SupportedStreamConfig| c.sample_rate() as i32)
            .unwrap_or(-1)
    }

    pub fn default_output_channels() -> i32 {
        use cpal::traits::HostTrait;
        cpal::default_host()
            .default_output_device()
            .and_then(|d: cpal::Device| d.default_output_config().ok())
            .map(|c: cpal::SupportedStreamConfig| i32::from(c.channels()))
            .unwrap_or(-1)
    }

    pub fn output_device_count() -> i32 {
        DeviceManager::new().output_devices().len() as i32
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.device_watcher_stop.store(true, Ordering::Relaxed);
        self.stop();
    }
}

// ── Free functions (module-private) ──────────────────────────────────────────

/// Open a `MediaSource` from a path string *or* URL string.
fn open_media_source(path: &str) -> Result<BoxedMediaSource, String> {
    if path.starts_with("http://") || path.starts_with("https://") {
        let s = HttpStream::new(path)?;
        Ok(Box::new(s))
    } else {
        let f = File::open(path)
            .map_err(|e| format!("Failed to open file '{path}': {e}"))?;
        Ok(Box::new(f))
    }
}

/// Build a `BoxedMediaSource` from an owned [`AudioSource`].
fn media_source_from_owned(source: AudioSource) -> Result<BoxedMediaSource, String> {
    match source {
        AudioSource::Path(p) => {
            let f = File::open(&p)
                .map_err(|e| format!("Failed to open file '{p}': {e}"))?;
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
            let f = File::open(p)
                .map_err(|e| format!("Failed to open source for duration: {e}"))?;
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
    let mut file = tempfile()
        .map_err(|e| format!("Failed to create temporary file for {tag}: {e}"))?;
    file.write_all(bytes)
        .map_err(|e| format!("Failed to write temporary file for {tag}: {e}"))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to rewind temporary file for {tag}: {e}"))?;
    Ok(file)
}

/// Probe a source for duration without decoding.
/// Returns `-1` if the duration cannot be determined.
fn estimate_duration_millis(source: &AudioSource, _out_channels: u32) -> i32 {
    let media = match media_source_from_ref(source) {
        Ok(m) => m,
        Err(e) => { error!("{e}"); return -1; }
    };

    let mss    = MediaSourceStream::new(media, MediaSourceStreamOptions::default());
    let probed = match symphonia::default::get_probe()
        .format(&Hint::new(), mss, &FormatOptions::default(), &MetadataOptions::default())
    {
        Ok(p) => p,
        Err(e) => { error!("Failed to probe source for duration: {e}"); return -1; }
    };

    let track = probed
        .format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL);

    let Some(track) = track else { return -1 };
    let cp = &track.codec_params;
    if let (Some(nf), Some(sr)) = (cp.n_frames, cp.sample_rate) {
        ((nf as f64 / sr as f64) * 1000.0) as i32
    } else {
        -1
    }
}

/// Decode packets from `source`, resample/remix to the output format, and
/// push interleaved `f32` chunks into `shared.queue`.
///
/// Runs on a dedicated background thread; terminates when `stop_flag` is set,
/// the source is exhausted, or an unrecoverable error occurs.
fn decode_and_feed(
    source:          AudioSource,
    stop_flag:       Arc<AtomicBool>,
    shared:          Arc<Mutex<SharedPlayback>>,
    out_channels:    usize,
    out_sample_rate: u32,
    start_millis:    i32,
) -> Result<(), String> {
    let media  = media_source_from_owned(source)?;
    let mss    = MediaSourceStream::new(media, MediaSourceStreamOptions::default());
    let probed = symphonia::default::get_probe()
        .format(&Hint::new(), mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| format!("Failed to probe audio format: {e}"))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No decodable audio track found")?
        .clone();

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Failed to create decoder: {e}"))?;

    let initial_playback_rate = shared
        .lock()
        .map(|s| s.playback_rate.clamp(MIN_RATE, MAX_RATE))
        .unwrap_or(1.0);

    let mut resample_state = ResampleState::new();
    let mut skip_output_samples = source_millis_to_output_samples(
        start_millis,
        out_sample_rate,
        out_channels,
        initial_playback_rate,
    );

    let decode_result = loop {
        if stop_flag.load(Ordering::SeqCst) { break Ok(()); }

        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::ResetRequired) =>
                break Err("Decoder reset required and not supported".to_string()),
            Err(SymphoniaError::IoError(_)) => break Ok(()),
            Err(e) => break Err(format!("Failed to read next packet: {e}")),
        };

        let decoded = match decoder.decode(&packet) {
            Ok(b) => b,
            Err(SymphoniaError::DecodeError(e)) => {
                warn!("Decode error: {e}. Skipping packet.");
                continue;
            }
            Err(SymphoniaError::IoError(_)) => break Ok(()),
            Err(e) => break Err(format!("Failed to decode packet: {e}")),
        };

        let (src_ch, src_rate, interleaved) = decoded_to_interleaved_f32(decoded);
        if interleaved.is_empty() || src_ch == 0 || src_rate == 0 { continue; }

        if stop_flag.load(Ordering::SeqCst) { break Ok(()); }

        let playback_rate = shared
            .lock()
            .map(|s| s.playback_rate.clamp(MIN_RATE, MAX_RATE))
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

        if out.is_empty() { continue; }

        let mut start = 0usize;
        if skip_output_samples > 0 {
            let consumed = skip_output_samples.min(out.len());
            skip_output_samples -= consumed;
            start = consumed;
        }
        if start >= out.len() { continue; }

        let out_slice = &out[start..];
        let mut offset = 0;

        while offset < out_slice.len() {
            if stop_flag.load(Ordering::SeqCst) { break; }

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
        s.stream_finished = true;
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
    ((start_millis.max(0) as f64)
        * out_sample_rate as f64
        * out_channels as f64
        / 1000.0
        / playback_rate as f64) as usize
}

// ── Sample format conversion helpers ─────────────────────────────────────────

/// Convert a Symphonia decoded buffer to interleaved `f32`.
fn decoded_to_interleaved_f32(decoded: AudioBufferRef<'_>) -> (usize, u32, Vec<f32>) {
    let spec     = *decoded.spec();
    let channels = spec.channels.count();
    let rate     = spec.rate;
    let mut buf  = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
    buf.copy_interleaved_ref(decoded);
    (channels, rate, buf.samples().to_vec())
}

/// Select the correct source sample for a given output channel.
///
/// Handles mono→stereo up-mix, stereo→mono down-mix, and channel remapping.
#[inline(always)]
fn source_frame_sample(
    frames:       &[f32],
    src_channels: usize,
    frame:        usize,
    out_channel:  usize,
    out_channels: usize,
) -> f32 {
    if src_channels == 0 { return 0.0; }
    let base = frame * src_channels;

    // Stereo → mono.
    if out_channels == 1 && src_channels > 1 {
        let acc: f32 = (0..src_channels).map(|c| frames[base + c]).sum();
        return acc / src_channels as f32;
    }

    // Mono → any.
    if src_channels == 1 { return frames[base]; }

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
    src_channels:    usize,
    src_rate:        u32,
    out_channels:    usize,
    out_rate:        u32,
    state:           &mut ResampleState,
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

    let estimated = (((total_frames as f64 - pos - 1.0) / step).max(0.0).ceil() as usize)
        .saturating_add(1);
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

    // Keep the last source frame as the left neighbour for the next packet.
    let keep_frame = total_frames - 1;
    let keep_base  = keep_frame * src_channels;
    state.carry.clear();
    state.carry.extend_from_slice(&frames[keep_base..keep_base + src_channels]);
    state.pos = pos - keep_frame as f64;

    out
}

// ── cpal output callbacks ─────────────────────────────────────────────────────

fn write_output_f32(data: &mut [f32], channels: usize, shared: &Arc<Mutex<SharedPlayback>>) {
    let mut g = match shared.lock() {
        Ok(g) => g,
        Err(_) => { debug!("Lock failed; filling silence"); data.fill(0.0); return; }
    };
    for frame in data.chunks_mut(channels) {
        for out in frame.iter_mut() {
            *out = g.next_sample();
        }
    }
}

fn write_output_i16(data: &mut [i16], channels: usize, shared: &Arc<Mutex<SharedPlayback>>) {
    let mut g = match shared.lock() {
        Ok(g) => g,
        Err(_) => { data.fill(0); return; }
    };
    for frame in data.chunks_mut(channels) {
        for out in frame.iter_mut() {
            *out = (g.next_sample() * i16::MAX as f32) as i16;
        }
    }
}

fn write_output_u16(data: &mut [u16], channels: usize, shared: &Arc<Mutex<SharedPlayback>>) {
    let mut g = match shared.lock() {
        Ok(g) => g,
        Err(_) => { data.fill(u16::MAX / 2); return; }
    };
    for frame in data.chunks_mut(channels) {
        for out in frame.iter_mut() {
            *out = (((g.next_sample() * 0.5) + 0.5) * u16::MAX as f32) as u16;
        }
    }
}

// ── Legacy free function shims (called from ffi.rs) ─────────────────────────

pub fn default_output_sample_rate() -> i32 { AudioEngine::default_output_sample_rate() }
pub fn default_output_channels()    -> i32 { AudioEngine::default_output_channels() }
pub fn output_device_count()        -> i32 { AudioEngine::output_device_count() }