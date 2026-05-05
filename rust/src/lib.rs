#![allow(unexpected_cfgs, dead_code)]

// ── Core modules ──────────────────────────────────────────────────────────────
mod error;       // Typed AudioError
mod events;      // AudioEvent + EventSender/Receiver
mod source;      // AudioSource enum
mod enums;       // Shared constants

// ── Engine layer ──────────────────────────────────────────────────────────────
mod device;      // DeviceManager + hotplug watcher
mod player_state; // SharedPlayback, PlaybackStatus, ResampleState
mod effects;     // AudioProcessor trait + Effects chain + built-in processors
mod processor;   // VisualizerProcessor (FFT spectrum)
mod http_stream; // HTTP/HTTPS MediaSource adapter

// ── Engine ────────────────────────────────────────────────────────────────────
mod engine;      // AudioEngine — ties everything together

// ── Logging macros ────────────────────────────────────────────────────────────
mod log;

// ── C / JNI FFI surface ───────────────────────────────────────────────────────
mod ffi;

// ── Android JNI bootstrap ─────────────────────────────────────────────────────
#[cfg(target_os = "android")]
mod android_init {
    use std::ffi::c_void;

    use jni::objects::{GlobalRef, JClass, JObject};
    use jni::JNIEnv;
    use once_cell::sync::OnceCell;

    static ANDROID_APP_CONTEXT: OnceCell<GlobalRef> = OnceCell::new();

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_thugbn_audiopc_AudiopcBridge_nativeInit(
        mut env: JNIEnv,
        _class: JClass,
        context: JObject,
    ) {
        let vm_ptr = match env.get_java_vm() {
            Ok(vm) => vm.get_java_vm_pointer() as *mut c_void,
            Err(_) => return,
        };

        if vm_ptr.is_null() {
            return;
        }

        let context_ptr = if let Some(existing) = ANDROID_APP_CONTEXT.get() {
            existing.as_obj().as_raw() as *mut c_void
        } else {
            let global = match env.new_global_ref(context) {
                Ok(g) => g,
                Err(_) => return,
            };
            let raw = global.as_obj().as_raw() as *mut c_void;
            let _ = ANDROID_APP_CONTEXT.set(global);
            raw
        };

        if context_ptr.is_null() {
            return;
        }

        // SAFETY: vm_ptr/context_ptr come from JVM-provided handles.
        unsafe {
            ndk_context::initialize_android_context(vm_ptr, context_ptr);
        }
    }
}