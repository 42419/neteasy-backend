//! neteasy-backend: embedded NetEase Cloud Music API backend, a Rust port of the endpoints
//! of `NeteaseCloudMusicApiEnhanced` (api-enhanced) that the neteasy app actually uses.
//!
//! Layout deliberately mirrors upstream so changes can be ported file-by-file:
//!   util/             <- util/            (crypto, request, option, config, xeapi_key, index helpers)
//!   module/           <- module/          (one file per endpoint; file name == route with `/` -> `_`)
//!   server            <- server.js        (HTTP routing, query/cookie handling, Set-Cookie)
//!   generate_config   <- generateConfig.js (anonymous token + xeapi public key bootstrap)
pub mod module;
pub mod util;
