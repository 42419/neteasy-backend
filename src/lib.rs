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

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct ServeOptions {
    pub host: String,
    pub port: u16,
    /// 设备 ID、xeapi 公钥、游客 token 在两次运行之间的缓存目录。
    pub data_dir: PathBuf,
    /// GET 响应缓存的有效期（秒），0 表示关闭。默认 120，对应上游的 `2 minutes`。
    pub cache_ttl_secs: u64,
    /// 是否一律用随机中国 IP 填充 X-Real-IP（对应上游环境变量 `ENABLE_RANDOM_CN_IP`）。
    /// 默认关闭：国内网络不需要；单次请求仍可用 `randomCNIP=true` 开启。
    pub random_cn_ip: bool,
}

impl Default for ServeOptions {
    fn default() -> Self {
        ServeOptions {
            host: "127.0.0.1".into(),
            port: 19800,
            data_dir: PathBuf::from("./data"),
            cache_ttl_secs: 120,
            random_cn_ip: false,
        }
    }
}

/// 对应上游的 `serveNcmApi`：先绑定端口（App 立刻就能连上），再在后台做引导初始化。
pub async fn serve_ncm_api(opts: ServeOptions) -> std::io::Result<()> {
    serve_with_ready(opts, None).await
}

/// 与 [`serve_ncm_api`] 相同，但端口绑定成功（或失败）后会通过 `ready` 通知调用方：
/// 成功时带上实际监听的地址（端口填 0 时可以由此得知系统分配的端口），失败时带上错误
/// （比如端口被占用）。这样 JNI 入口可以等到端口真正可连接再返回。
pub async fn serve_with_ready(
    opts: ServeOptions,
    ready: Option<mpsc::Sender<std::io::Result<SocketAddr>>>,
) -> std::io::Result<()> {
    let request = util::request::Request::new(&opts.data_dir);
    request.set_default_random_cn_ip(opts.random_cn_ip);
    let bound = match tokio::net::TcpListener::bind((opts.host.as_str(), opts.port)).await {
        Ok(l) => l.local_addr().map(|addr| (l, addr)),
        Err(e) => Err(e),
    };
    let (listener, addr) = match bound {
        Ok(v) => v,
        Err(e) => {
            if let Some(tx) = &ready {
                let _ = tx.send(Err(std::io::Error::new(e.kind(), e.to_string())));
            }
            return Err(e);
        }
    };
    if let Some(tx) = &ready {
        let _ = tx.send(Ok(addr));
    }
    log::info!("neteasy-backend listening on http://{addr}");
    let bootstrap = request.clone();
    tokio::spawn(async move { generate_config::generate_config(&bootstrap).await });
    let app = server::build_app_with(request, Duration::from_secs(opts.cache_ttl_secs))
        .into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, app).await
}

/// 在独立线程里运行的后端服务句柄，可以主动停止。
///
/// 丢弃句柄（Drop）也会通知服务停止，但不会等待线程退出；需要确认已停止时用 [`ServerHandle::stop`]。
pub struct ServerHandle {
    addr: SocketAddr,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl ServerHandle {
    /// 实际监听的地址
    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }

    /// 停止服务并等待线程退出（进行中的请求会被直接丢弃，最多再等 1 秒回收任务）。
    pub fn stop(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

/// 在新线程里启动服务，端口绑定成功后才返回（最多等 5 秒）。
/// 绑定失败（如端口被占用）时返回对应的错误。
pub fn spawn_server(opts: ServeOptions) -> std::io::Result<ServerHandle> {
    let (ready_tx, ready_rx) = mpsc::channel();
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let thread = std::thread::Builder::new()
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
                    let _ = ready_tx.send(Err(e));
                    return;
                }
            };
            rt.block_on(async move {
                tokio::select! {
                    r = serve_with_ready(opts, Some(ready_tx)) => {
                        if let Err(e) = r {
                            log::error!("server stopped: {e}");
                        }
                    }
                    // 收到信号，或句柄被丢弃（发送端关闭）都表示要停止
                    _ = stop_rx => log::info!("收到停止信号，后端退出"),
                }
            });
            // 强制回收还在跑的连接任务
            rt.shutdown_timeout(Duration::from_secs(1));
        })?;
    match ready_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(addr)) => Ok(ServerHandle {
            addr,
            shutdown: Some(stop_tx),
            thread: Some(thread),
        }),
        Ok(Err(e)) => {
            let _ = thread.join();
            Err(e)
        }
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "等待后端绑定端口超时",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!("neteasy-lib-{}", rand::random::<u32>()))
    }

    fn opts(port: u16) -> ServeOptions {
        ServeOptions {
            host: "127.0.0.1".into(),
            port,
            data_dir: temp_dir(),
            ..Default::default()
        }
    }

    /// 最简单的 HTTP/1.1 GET，返回完整响应文本
    fn http_get(addr: SocketAddr, path: &str) -> std::io::Result<String> {
        let mut s = std::net::TcpStream::connect_timeout(&addr, Duration::from_secs(2))?;
        s.set_read_timeout(Some(Duration::from_secs(5)))?;
        write!(
            s,
            "GET {path} HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n"
        )?;
        let mut out = String::new();
        s.read_to_string(&mut out)?;
        Ok(out)
    }

    #[tokio::test]
    async fn ready_is_signalled_after_bind() {
        let (tx, rx) = mpsc::channel();
        let handle = tokio::spawn(serve_with_ready(opts(0), Some(tx)));
        let got = tokio::task::spawn_blocking(move || rx.recv_timeout(Duration::from_secs(5)))
            .await
            .unwrap();
        let addr = got.expect("recv").expect("bind");
        assert_ne!(addr.port(), 0, "应当返回系统分配的真实端口");
        handle.abort();
    }

    #[tokio::test]
    async fn bind_error_is_reported_through_ready() {
        // 先占住一个端口，再让服务去绑同一个
        let held = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = held.local_addr().unwrap().port();
        let (tx, rx) = mpsc::channel();
        let result = serve_with_ready(opts(port), Some(tx)).await;
        assert!(result.is_err());
        assert!(matches!(
            rx.recv_timeout(Duration::from_secs(1)),
            Ok(Err(_))
        ));
    }

    #[test]
    fn spawn_serve_stop_and_restart_on_the_same_port() {
        let handle = spawn_server(opts(0)).expect("start");
        let addr = handle.local_addr();
        let resp = http_get(addr, "/").expect("get");
        assert!(resp.starts_with("HTTP/1.1 200"), "{resp}");
        assert!(resp.contains("neteasy-backend"));

        handle.stop();
        assert!(
            std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_err(),
            "停止后端口应当不再可连接"
        );

        // 同一个端口可以立刻再次启动
        let again = spawn_server(opts(addr.port())).expect("restart");
        assert!(http_get(again.local_addr(), "/")
            .expect("get")
            .contains("neteasy-backend"));
        again.stop();
    }

    #[test]
    fn spawn_reports_port_in_use() {
        let held = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = held.local_addr().unwrap().port();
        let err = spawn_server(opts(port)).err().expect("应当失败");
        assert_eq!(err.kind(), std::io::ErrorKind::AddrInUse);
    }

    #[test]
    fn dropping_the_handle_stops_the_server() {
        let handle = spawn_server(opts(0)).expect("start");
        let addr = handle.local_addr();
        drop(handle);
        // Drop 只发信号不等待，给后台线程一点时间退出
        let mut closed = false;
        for _ in 0..40 {
            if std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_err() {
                closed = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(closed, "丢弃句柄后服务应当停止");
    }
}
