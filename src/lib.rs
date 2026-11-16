//! neteasy-backend: embedded NetEase Cloud Music API backend, a Rust port of the endpoints
//! of `NeteaseCloudMusicApiEnhanced` (api-enhanced) that the neteasy app actually uses.
//!
//! Layout deliberately mirrors upstream so changes can be ported file-by-file:
//!   util/             <- util/            (crypto, request, option, config, xeapi_key, index helpers)
//!   module/           <- module/          (one file per endpoint; file name == route with `/` -> `_`)
//!   server            <- server.js        (HTTP routing, query/cookie handling, Set-Cookie)
//!   generate_config   <- generateConfig.js (anonymous token + xeapi public key bootstrap)
// Modules return `Result<Answer, Answer>` like upstream's resolve/reject of `{ status, body, cookie }`.
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
    /// Where device id / xeapi key / anonymous token are cached between runs.
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

/// Upstream `serveNcmApi`: bind first (so the app can connect immediately), bootstrap in the background.
pub async fn serve_ncm_api(opts: ServeOptions) -> std::io::Result<()> {
    let request = util::request::Request::new(&opts.data_dir);
    let listener = tokio::net::TcpListener::bind((opts.host.as_str(), opts.port)).await?;
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
