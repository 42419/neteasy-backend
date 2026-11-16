//! Android 的 JNI 入口（cdylib：`libneteasy_backend.so`）。
//!
//! Kotlin 端写法：
//! ```kotlin
//! object RustBackend {
//!     init { System.loadLibrary("neteasy_backend") }
//!     @JvmStatic external fun start(dataDir: String, port: Int): Boolean
//! }
//! ```
//! 类名必须是 `top.yunov.neteasy.RustBackend`（JNI 符号名里编码了类名）。

use crate::{serve_with_ready, ServeOptions};
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use std::sync::atomic::{AtomicBool, Ordering};

static STARTED: AtomicBool = AtomicBool::new(false);

/// 在后台线程启动 HTTP 后端，并等到端口绑定成功后才返回（最多等 5 秒）。
/// 返回 `true` 表示本次调用成功启动、端口已可连接；已经在运行、参数无效或绑定失败（如端口被占用）时返回 `false`。
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
    let (tx, rx) = std::sync::mpsc::channel();
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
                    let _ = tx.send(Err(e));
                    STARTED.store(false, Ordering::SeqCst);
                    return;
                }
            };
            if let Err(e) = rt.block_on(serve_with_ready(opts, Some(tx))) {
                log::error!("server stopped: {e}");
            }
            STARTED.store(false, Ordering::SeqCst);
        });
    if spawned.is_err() {
        STARTED.store(false, Ordering::SeqCst);
        return JNI_FALSE;
    }
    match rx.recv_timeout(std::time::Duration::from_secs(5)) {
        Ok(Ok(())) => JNI_TRUE,
        _ => JNI_FALSE,
    }
}
