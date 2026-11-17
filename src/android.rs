//! Android 的 JNI 入口（cdylib：`libneteasy_backend.so`）。
//!
//! Kotlin 端写法：
//! ```kotlin
//! object RustBackend {
//!     init { System.loadLibrary("neteasy_backend") }
//!     @JvmStatic external fun start(dataDir: String, port: Int): Boolean
//!     @JvmStatic external fun stop(): Boolean
//!     @JvmStatic external fun isRunning(): Boolean
//! }
//! ```
//! 类名必须是 `top.yunov.neteasy.RustBackend`（JNI 符号名里编码了类名）。

use crate::{spawn_server, ServeOptions, ServerHandle};
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use std::sync::Mutex;

/// 当前运行中的后端（同一时刻只会有一个）
static HANDLE: Mutex<Option<ServerHandle>> = Mutex::new(None);

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
    if !(1..=65535).contains(&port) {
        return JNI_FALSE;
    }
    let mut guard = HANDLE.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_some() {
        return JNI_FALSE;
    }
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("neteasy-backend"),
    );
    let opts = ServeOptions {
        port: port as u16,
        data_dir: dir.into(),
        ..Default::default()
    };
    match spawn_server(opts) {
        Ok(handle) => {
            *guard = Some(handle);
            JNI_TRUE
        }
        Err(e) => {
            log::error!("启动失败: {e}");
            JNI_FALSE
        }
    }
}

/// 停止后端并等待线程退出；之后可以再次调用 `start`。
/// 返回 `true` 表示确实停掉了一个正在运行的后端，没在运行时返回 `false`。
#[no_mangle]
pub extern "system" fn Java_top_yunov_neteasy_RustBackend_stop(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    // 先把句柄取出来再停止，避免停止期间一直占着锁
    let handle = HANDLE.lock().unwrap_or_else(|e| e.into_inner()).take();
    match handle {
        Some(h) => {
            h.stop();
            JNI_TRUE
        }
        None => JNI_FALSE,
    }
}

/// 后端当前是否在运行。
#[no_mangle]
pub extern "system" fn Java_top_yunov_neteasy_RustBackend_isRunning(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    if HANDLE.lock().unwrap_or_else(|e| e.into_inner()).is_some() {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}
