pub mod api;
mod frb_generated;

#[cfg(target_os = "android")]
mod init_android_context {
    use jni::{
        JNIEnv,
        objects::{GlobalRef, JClass, JObject},
    };
    use std::ffi::c_void;
    use std::sync::OnceLock;

    static CTX: OnceLock<GlobalRef> = OnceLock::new();

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_thugbn_audiopc_AudiopcPlugin_init_1android(
        env: JNIEnv,
        _class: JClass,
        ctx: JObject,
    ) {
        if CTX.get().is_some() {
            log::info!("Android context already initialized; skipping");
        } else {
            log::info!("Initializing Android context");
        }
        // Initialize the process-level Android context only once.
        CTX.get_or_init(|| {
            let global_ref = env
                .new_global_ref(&ctx)
                .expect("Failed to create Android global reference");

            let vm = env.get_java_vm().expect("Failed to get JavaVM");

            let vm = vm.get_java_vm_pointer() as *mut c_void;

            unsafe {
                ndk_context::initialize_android_context(vm, global_ref.as_obj().as_raw() as _);
            }

            global_ref
        });

        flutter_rust_bridge::setup_default_user_utils();
    }
}
