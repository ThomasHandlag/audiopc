use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Mutex;

use log::error;
use once_cell::sync::Lazy;

use crate::engine::{
    default_output_channels, default_output_sample_rate, output_device_count, AudioEngine,
    AudioSource,
};

static ENGINE: Lazy<Mutex<Option<AudioEngine>>> = Lazy::new(|| Mutex::new(None));

fn with_engine_mut<F>(mut f: F) -> i32
where
    F: FnMut(&mut AudioEngine) -> Result<(), String>,
{
    let mut guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => {
            error!("Engine mutex is poisoned");
            return -500;
        }
    };

    if guard.is_none() {
        match AudioEngine::new() {
            Ok(engine) => {
                *guard = Some(engine);
            }
            Err(err) => {
                error!("Engine initialization failed: {err}");
                return -501;
            }
        }
    }

    let Some(engine) = guard.as_mut() else {
        error!("Engine initialization failed: missing engine state");
        return -502;
    };

    match f(engine) {
        Ok(()) => 0,
        Err(err) => {
            error!("FFI operation failed: {err}");
            -1
        }
    }
}

fn with_engine_ref<F>(mut f: F) -> i32
where
    F: FnMut(&AudioEngine) -> i32,
{
    let guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return -500,
    };

    let Some(engine) = guard.as_ref() else {
        return 0;
    };

    f(engine)
}

fn c_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    // SAFETY: Caller promises ptr is a valid, NUL-terminated C string.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().ok().map(ToOwned::to_owned)
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_default_output_sample_rate() -> i32 {
    default_output_sample_rate()
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_default_output_channels() -> i32 {
    default_output_channels()
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_output_device_count() -> i32 {
    output_device_count()
}

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
    let slice = unsafe { std::slice::from_raw_parts(data, len as usize) };
    let memory_data = slice.to_vec();

    with_engine_mut(|engine| {
        engine.set_source(AudioSource::Memory(memory_data.clone()));
        Ok(())
    })
}

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

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_volume(volume: f64) -> i32 {
    with_engine_mut(|engine| {
        engine.set_volume(volume as f32);
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_set_lowpass_hz(hz: f64) -> i32 {
    with_engine_mut(|engine| {
        engine.set_lowpass_hz(hz as f32);
        Ok(())
    })
}

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
pub extern "C" fn audiopc_is_playing() -> i32 {
    with_engine_ref(|engine| engine.is_playing())
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_is_source_loaded() -> i32 {
    with_engine_ref(|engine| engine.is_source_loaded())
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_duration_millis() -> i32 {
    with_engine_ref(|engine| engine.duration_millis()
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_position_millis() -> i32 {
    with_engine_ref(|engine| engine.position_millis()
)
}

#[unsafe(no_mangle)]
pub extern "C" fn audiopc_seek_millis(millis: i32) {
    with_engine_mut(|engine| {
        engine.seek(millis);
        Ok(())
    });
}

fn with_engine_mut_i32<F>(mut f: F) -> i32
where
    F: FnMut(&mut AudioEngine) -> Result<i32, String>,
{
    let mut guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => {
            error!("Engine mutex is poisoned");
            return -500;
        }
    };

    if guard.is_none() {
        match AudioEngine::new() {
            Ok(engine) => {
                *guard = Some(engine);
            }
            Err(err) => {
                error!("Engine initialization failed: {err}");
                return -501;
            }
        }
    }

    let Some(engine) = guard.as_mut() else {
        error!("Engine initialization failed: missing engine state");
        return -502;
    };

    match f(engine) {
        Ok(value) => value,
        Err(err) => {
            error!("FFI operation failed: {err}");
            -1
        }
    }
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