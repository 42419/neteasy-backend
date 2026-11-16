//! Android JNI entry (cdylib `libneteasy_backend.so`).
//!
//! Kotlin side:
//! ```kotlin
//! object RustBackend {
//!     init { System.loadLibrary("neteasy_backend") }
//!     @JvmStatic external fun start(dataDir: String, port: Int): Boolean
//! }
//! ```
//! The class must be `top.yunov.neteasy.RustBackend` (the JNI symbol name encodes it).

use crate::{serve_ncm_api, ServeOptions};
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use std::sync::atomic::{AtomicBool, Ordering};

static STARTED: AtomicBool = AtomicBool::new(false);

/// Starts the HTTP backend on a background thread and returns immediately.
/// Returns `true` when this call started it, `false` if it was already running or the arguments were invalid.
#[no_mangle]
pub extern "system" fn Java_top_yunov_neteasy_RustBackend_start(
    mut env: JNIEnv,
    _class: JClass,
    data_dir: JString,
    port: jint,
) -> jboolean {
    let dir: String = match env.get_string(&data_dir) {
        Ok(s) => s.into(),
        Err(_) => return JNI_FALSE,
    };
    if !(1..=65535).contains(&port)
        || STARTED
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
    {
        return JNI_FALSE;
    }
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("neteasy-backend"),
    );
    let opts = ServeOptions {
        host: "127.0.0.1".into(),
        port: port as u16,
        data_dir: dir.into(),
    };
    let spawned = std::thread::Builder::new()
        .name("neteasy-backend".into())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    log::error!("tokio runtime: {e}");
                    STARTED.store(false, Ordering::SeqCst);
                    return;
                }
            };
            if let Err(e) = rt.block_on(serve_ncm_api(opts)) {
                log::error!("server stopped: {e}");
            }
            STARTED.store(false, Ordering::SeqCst);
        });
    if spawned.is_err() {
        STARTED.store(false, Ordering::SeqCst);
        return JNI_FALSE;
    }
    JNI_TRUE
}
