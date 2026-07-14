/// C-compatible FFI surface for the audio engine.
///
/// All symbols are exported with `#[unsafe(no_mangle)]` and `extern "C"` so
/// they can be called from Dart (via `dart:ffi`), Java (via JNI), or any
/// other C-compatible caller.
///
/// # Error codes
///
/// Functions that can fail return an `i32`:
/// * `0`    — success.
/// * `> 0`  — positive result (e.g., byte/sample count).
/// * `-1`   — general engine / operation error.
/// * `-2`   — bad argument (null pointer, invalid range, …).
/// * `-500` — engine mutex poisoned.
/// * `-501` — engine failed to initialise.
/// * `-502` — engine reference missing after init (internal bug).

use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::{
    engine::AudioEngine,
    error, info,
    player_state::PlayerState,
    source::AudioSource,
};

// ── Singleton engine ──────────────────────────────────────────────────────────

static ENGINE: Lazy<Mutex<Option<AudioEngine>>> = Lazy::new(|| Mutex::new(None));

/// Rebuild the cpal stream after a device error.
///
/// Called from the stream's error callback via a background thread.
pub fn revise_stream() {
    with_engine_mut(|engine| {
        info!("Revising stream ...");
        engine.reset_stream()
    });
}

// ── Internal dispatch helpers ─────────────────────────────────────────────────

/// Acquire the engine (initialising it on first call) and run `f`.
/// `R` must implement `Default` so we have a sentinel value for failures.
fn with_engine<F, R>(mut f: F) -> R
where
    F: FnMut(&mut AudioEngine) -> R,
    R: Default,
{
    let mut guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => { error!("Engine mutex is poisoned"); return R::default(); }
    };

    if guard.is_none() {
        match AudioEngine::new() {
            Ok(eng) => { *guard = Some(eng); }
            Err(err) => { error!("Engine initialization failed: {err}"); return R::default(); }
        }
    }

    let Some(engine) = guard.as_mut() else {
        error!("Engine initialization failed: missing engine state");
        return R::default();
    };

    f(engine)
}

/// Like `with_engine` but maps `Result<(), String>` → `i32` (0 ok, -1 err).
fn with_engine_mut<F>(mut f: F) -> i32
where
    F: FnMut(&mut AudioEngine) -> Result<(), String>,
{
    let mut guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => { error!("Engine mutex is poisoned"); return -500; }
    };

    if guard.is_none() {
        match AudioEngine::new() {
            Ok(eng) => { *guard = Some(eng); }
            Err(err) => { error!("Engine initialization failed: {err}"); return -501; }
        }
    }

    let Some(engine) = guard.as_mut() else {
        error!("Engine initialization failed: missing engine state");
        return -502;
    };

    match f(engine) {
        Ok(())  => 0,
        Err(e)  => { error!("FFI operation failed: {e}"); -1 }
    }
}

/// Like `with_engine` but the closure returns `i32` directly.
fn with_engine_ref<F>(mut f: F) -> i32
where
    F: FnMut(&AudioEngine) -> i32,
{
    let guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return -500,
    };

    let Some(engine) = guard.as_ref() else { return -502; };
    f(engine)
}

/// Like `with_engine_mut` but the closure returns `Result<i32, String>`.
fn with_engine_mut_i32<F>(mut f: F) -> i32
where
    F: FnMut(&mut AudioEngine) -> Result<i32, String>,
{
    let mut guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => { error!("Engine mutex is poisoned"); return -500; }
    };

    if guard.is_none() {
        match AudioEngine::new() {
            Ok(eng) => { *guard = Some(eng); }
            Err(err) => { error!("Engine initialization failed: {err}"); return -501; }
        }
    }

    let Some(engine) = guard.as_mut() else {
        error!("Engine initialization failed: missing engine state");
        return -502;
    };

    match f(engine) {
        Ok(v)  => v,
        Err(e) => { error!("FFI operation failed: {e}"); -1 }
    }
}

// ── C string helper ───────────────────────────────────────────────────────────

fn c_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() { return None; }
    // SAFETY: Caller promises `ptr` is a valid, NUL-terminated C string.
    unsafe { CStr::from_ptr(ptr) }.to_str().ok().map(ToOwned::to_owned)
}

// ── Device query ──────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_default_output_sample_rate() -> i32 {
    AudioEngine::default_output_sample_rate()
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_default_output_channels() -> i32 {
    AudioEngine::default_output_channels()
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_output_device_count() -> i32 {
    AudioEngine::output_device_count()
}

// ── Source selection ──────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_source_path(path: *const c_char) -> i32 {
    let Some(path) = c_string(path) else {
        error!("Source path is null or invalid UTF-8");
        return -2;
    };

    if std::fs::File::open(&path).is_err() {
        error!("Could not open source file: {path}");
        return -3;
    }

    with_engine_mut(|engine| {
        engine.set_source(AudioSource::Path(path.clone()));
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_source_url(url: *const c_char) -> i32 {
    let Some(url) = c_string(url) else {
        error!("Source URL is null or invalid UTF-8");
        return -2;
    };

    with_engine_mut(|engine| {
        engine.set_source(AudioSource::Url(url.clone()));
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_source_memory(data: *const u8, len: i32) -> i32 {
    if data.is_null() || len <= 0 {
        error!("Source memory pointer is null or length is non-positive");
        return -2;
    }

    // SAFETY: Caller must provide a valid pointer for `len` bytes.
    let bytes = unsafe { std::slice::from_raw_parts(data, len as usize) }.to_vec();

    with_engine_mut(|engine| {
        engine.set_source(AudioSource::Memory(bytes.clone()));
        Ok(())
    })
}

// ── Playback control ──────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_play() -> i32 {
    with_engine_mut(|engine| {
        engine.ensure_stream()?;
        engine.start_decode_thread_if_needed()?;
        engine.set_playing(true);
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_pause() -> i32 {
    with_engine_mut(|engine| {
        engine.set_playing(false);
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_stop() -> i32 {
    with_engine_mut(|engine| {
        engine.stop();
        Ok(())
    })
}

// ── Volume / rate ─────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_volume(volume: f64) -> i32 {
    with_engine_mut(|engine| {
        engine.set_volume(volume as f32);
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_rate(rate: f32) -> i32 {
    with_engine_mut_i32(|engine| {
        engine.set_rate(rate);
        Ok(0)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_get_rate() -> f32 {
    with_engine(|engine| engine.rate())
}

// ── Queue / buffering ─────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_max_queue_seconds(seconds: i32) -> i32 {
    if seconds <= 0 {
        error!("max queue seconds must be positive");
        return -2;
    }
    with_engine_mut(|engine| {
        engine.set_max_queue_seconds(seconds as usize);
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_get_max_queue_seconds() -> i32 {
    with_engine_ref(|engine| engine.max_queue_seconds())
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_buffered_samples() -> i32 {
    with_engine_ref(|engine| engine.buffered_samples())
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_buffered_millis() -> i32 {
    with_engine_ref(|engine| engine.buffered_millis())
}

// ── Seek / position / duration ────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_seek_millis(millis: i32) -> i32 {
    with_engine_mut(|engine| {
        engine.seek(millis);
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_duration_millis() -> i32 {
    with_engine_ref(|engine| engine.duration_millis())
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_position_millis() -> i32 {
    with_engine_ref(|engine| engine.position_millis())
}

// ── Player state ──────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_is_playing() -> i32 {
    with_engine_ref(|engine| engine.is_playing())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn audiopc_get_player_state() -> i32 {
    with_engine_ref(|engine| match engine.get_state() {
        PlayerState::Idle    => 0,
        PlayerState::Playing => 1,
        PlayerState::Paused  => 2,
        PlayerState::Stopped => 3,
    })
}

// ── Visualizer ────────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_visualizer_available_samples() -> i32 {
    with_engine_ref(|engine| engine.visualizer_available_samples())
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_visualizer_sample_rate() -> i32 {
    with_engine_ref(|engine| engine.visualizer_sample_rate())
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_visualizer_channels() -> i32 {
    with_engine_ref(|engine| engine.visualizer_channels())
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_copy_visualizer_samples(buffer: *mut f32, max_samples: i32) -> i32 {
    if buffer.is_null() || max_samples <= 0 {
        error!("Visualizer output buffer is null or max_samples is non-positive");
        return -2;
    }
    with_engine_ref(|engine| {
        // SAFETY: Caller provides a valid writable pointer for max_samples f32 values.
        let out = unsafe { std::slice::from_raw_parts_mut(buffer, max_samples as usize) };
        engine.copy_visualizer_samples(out)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_copy_visualizer_spectrum(buffer: *mut f32, max_bars: i32) -> i32 {
    if buffer.is_null() || max_bars <= 0 {
        error!("Visualizer spectrum buffer is null or max_bars is non-positive");
        return -2;
    }
    with_engine_mut_i32(|engine| {
        // SAFETY: Caller provides a valid writable pointer for max_bars f32 values.
        let out = unsafe { std::slice::from_raw_parts_mut(buffer, max_bars as usize) };
        Ok(engine.copy_visualizer_spectrum(out))
    })
}

// ── Metadata / thumbnail ──────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_get_metadata(
    buffer:  *mut c_char,
    max_len: i32,
    path:    *const c_char,
) -> i32 {
    if buffer.is_null() || max_len <= 0 {
        error!("Metadata buffer is null or max_len is non-positive");
        return -2;
    }

    let Some(path) = c_string(path) else {
        error!("Metadata path is null or invalid UTF-8");
        return -2;
    };

    with_engine_ref(|engine| {
        let json = match engine.get_metadata(&path) {
            Ok(j) => j,
            Err(e) => { error!("Failed to get metadata: {e}"); return -1; }
        };
        let bytes = json.as_bytes();
        let copy_len = bytes.len() as i32;

        if copy_len <= 0 || copy_len >= max_len {
            return -2; // Need space for the null terminator.
        }

        // SAFETY: buffer is valid and large enough (checked above).
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr() as *const c_char,
                buffer,
                copy_len as usize,
            );
            *buffer.add(copy_len as usize) = 0;
        }

        bytes.len() as i32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_get_thumbnail(
    buffer:  *mut u8,
    max_len: i32,
    path:    *const c_char,
) -> i32 {
    if buffer.is_null() || max_len <= 0 {
        error!("Thumbnail buffer is null or max_len is non-positive");
        return -2;
    }

    let Some(path) = c_string(path) else {
        error!("Thumbnail path is null or invalid UTF-8");
        return -2;
    };

    with_engine_ref(|engine| {
        let data = match engine.get_thumbnail(&path) {
            Ok(d) => d,
            Err(e) => { error!("Failed to get thumbnail: {e}"); return -1; }
        };

        let copy_len = data.len().min(max_len as usize);
        if copy_len > 0 {
            // SAFETY: buffer is valid for max_len bytes (caller contract).
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr(), buffer, copy_len);
            }
        }

        copy_len as i32
    })
}

// ── DSP filters ───────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_clear_filters() -> i32 {
    with_engine_mut(|engine| { engine.clear_filters(); Ok(()) })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_peak_filter(center_hz: f32, gain_db: f32, q: f32) -> i32 {
    with_engine_mut(|engine| { engine.set_peak_filter(center_hz, gain_db, q); Ok(()) })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_low_shelf_filter(cutoff_hz: f32, gain_db: f32, q: f32) -> i32 {
    with_engine_mut(|engine| { engine.set_low_shelf_filter(cutoff_hz, gain_db, q); Ok(()) })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_high_shelf_filter(cutoff_hz: f32, gain_db: f32, q: f32) -> i32 {
    with_engine_mut(|engine| { engine.set_high_shelf_filter(cutoff_hz, gain_db, q); Ok(()) })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_band_pass_filter(center_hz: f32, q: f32) -> i32 {
    with_engine_mut(|engine| { engine.set_band_pass_filter(center_hz, q); Ok(()) })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_notch_filter(center_hz: f32, q: f32) -> i32 {
    with_engine_mut(|engine| { engine.set_notch_filter(center_hz, q); Ok(()) })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_lowpass_hz(cutoff_hz: f64, q: f32) -> i32 {
    with_engine_mut(|engine| { engine.set_lowpass_filter(cutoff_hz as f32, q); Ok(()) })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_high_pass_filter(cutoff_hz: f32, q: f32) -> i32 {
    with_engine_mut(|engine| { engine.set_high_pass_filter(cutoff_hz, q); Ok(()) })
}