//! neteasy-backend：内嵌的网易云音乐 API 后端，是 `NeteaseCloudMusicApiEnhanced`（api-enhanced）
//! 中 neteasy 应用实际用到的接口的 Rust 移植版。
//!
//! 目录结构有意与上游保持一致，这样上游的改动可以按文件逐个对照移植：
//!   util/             <- util/            （crypto、request、option、config、xeapi_key、index 工具）
//!   module/           <- module/          （一个接口一个文件；文件名 == 路由里的 `/` 换成 `_`）
//!   server            <- server.js        （HTTP 路由、query/cookie 处理、Set-Cookie）
//!   generate_config   <- generateConfig.js （游客 token 与 xeapi 公钥的启动初始化）
// 模块返回 `Result<Answer, Answer>`，对应上游对 `{ status, body, cookie }` 的 resolve/reject。
#![allow(clippy::result_large_err)]

#[cfg(target_os = "android")]
mod android;
pub mod generate_config;
pub mod module;
pub mod server;
pub mod util;

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ServeOptions {
    pub host: String,
    pub port: u16,
    /// 设备 ID、xeapi 公钥、游客 token 在两次运行之间的缓存目录。
    pub data_dir: PathBuf,
}

impl Default for ServeOptions {
    fn default() -> Self {
        ServeOptions {
            host: "127.0.0.1".into(),
            port: 19800,
            data_dir: PathBuf::from("./data"),
        }
    }
}

/// 对应上游的 `serveNcmApi`：先绑定端口（App 立刻就能连上），再在后台做引导初始化。
pub async fn serve_ncm_api(opts: ServeOptions) -> std::io::Result<()> {
    serve_with_ready(opts, None).await
}

/// 与 [`serve_ncm_api`] 相同，但端口绑定成功（或失败）后会通过 `ready` 通知调用方，
/// 这样 JNI 入口可以等到端口真正可连接再返回，并把端口被占用之类的错误如实报告出去。
pub async fn serve_with_ready(
    opts: ServeOptions,
    ready: Option<std::sync::mpsc::Sender<std::io::Result<()>>>,
) -> std::io::Result<()> {
    let request = util::request::Request::new(&opts.data_dir);
    let listener = match tokio::net::TcpListener::bind((opts.host.as_str(), opts.port)).await {
        Ok(l) => l,
        Err(e) => {
            if let Some(tx) = &ready {
                let _ = tx.send(Err(std::io::Error::new(e.kind(), e.to_string())));
            }
            return Err(e);
        }
    };
    if let Some(tx) = &ready {
        let _ = tx.send(Ok(()));
    }
    log::info!(
        "neteasy-backend listening on http://{}:{}",
        opts.host,
        opts.port
    );
    let bootstrap = request.clone();
    tokio::spawn(async move { generate_config::generate_config(&bootstrap).await });
    axum::serve(
        listener,
        server::build_app(request).into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("neteasy-lib-{}", rand::random::<u32>()))
    }

    #[tokio::test]
    async fn ready_is_signalled_after_bind() {
        let (tx, rx) = std::sync::mpsc::channel();
        let opts = ServeOptions {
            host: "127.0.0.1".into(),
            port: 0,
            data_dir: temp_dir(),
        };
        let handle = tokio::spawn(serve_with_ready(opts, Some(tx)));
        let got =
            tokio::task::spawn_blocking(move || rx.recv_timeout(std::time::Duration::from_secs(5)))
                .await
                .unwrap();
        assert!(matches!(got, Ok(Ok(()))));
        handle.abort();
    }

    #[tokio::test]
    async fn bind_error_is_reported_through_ready() {
        // 先占住一个端口，再让服务去绑同一个
        let held = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = held.local_addr().unwrap().port();
        let (tx, rx) = std::sync::mpsc::channel();
        let opts = ServeOptions {
            host: "127.0.0.1".into(),
            port,
            data_dir: temp_dir(),
        };
        let result = serve_with_ready(opts, Some(tx)).await;
        assert!(result.is_err());
        assert!(matches!(
            rx.recv_timeout(std::time::Duration::from_secs(1)),
            Ok(Err(_))
        ));
    }
}
