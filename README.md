# neteasy-backend

English | [简体中文](README.zh-CN.md)

Rust port of the [NeteaseCloudMusicApiEnhanced](https://github.com/NeteaseCloudMusicApiEnhanced/api-enhanced)
(api-enhanced) endpoints used by the [neteasy](https://github.com/42419/neteasy) Android app.
Serves the same HTTP interface on `127.0.0.1:19800` so the app's `ApiClient` keeps working unchanged;
replaces the embedded `nodejs-mobile` backend (no JS runtime, no module-loading startup cost).

## Layout (mirrors api-enhanced)

| upstream               | here                      | responsibility                                        |
|------------------------|---------------------------|-------------------------------------------------------|
| `util/crypto.js`       | `src/util/crypto.rs`      | weapi / eapi / xeapi request + response crypto        |
| `util/request.js`      | `src/util/request.rs`     | request pipeline: cookies, headers, crypto dispatch   |
| `util/option.js`       | `src/util/option.rs`      | per-request options                                   |
| `util/config.json`     | `src/util/config.rs`      | domains / app constants                               |
| `util/xeapiKey.js`     | `src/util/xeapi_key.rs`   | xeapi public key + session state                      |
| `module/*.js`          | `src/module/*.rs`         | one file per endpoint; `song_url_v1` -> `/song/url/v1`|
| `server.js`            | `src/server.rs`           | HTTP routing, query/cookie handling, Set-Cookie       |
| `generateConfig.js`    | `src/generate_config.rs`  | anonymous token + xeapi key bootstrap                 |

Tracked upstream version: api-enhanced 4.41.1 (`75bf2e5`).

## Verification

`scripts/gen_vectors.js` runs upstream `util/crypto.js` with injected randomness and writes
`tests/vectors.json`; `tests/crypto_vectors.rs` asserts the Rust output is byte-identical
(weapi, eapi, xeapi incl. session-key path, response decrypt, sign, public-key blob).

```bash
cargo test
```

## Porting status

Endpoints the app calls (from `ApiClient` usage), with upstream crypto mode:

| route | crypto | status |
|-------|--------|--------|
| `/login/qr/key`, `/login/qr/create`, `/login/qr/check`, `/login/status` | eapi / local / eapi / weapi | ported, verified live on a China network |
| `/song/url/v1` | xeapi | ported, verified live on a China network |
| `/song/detail`, `/lyric`, `/lyric/new` | weapi / eapi | ported, verified live on a China network |
| `/search`, `/search/suggest`, `/search/hot` | eapi / weapi / eapi | ported, verified live on a China network |
| `/playlist/detail`, `/playlist/track/all`, `/user/playlist`, `/user/detail` | eapi / weapi | ported, verified live on a China network |
| `/recommend/songs`, `/personalized`, `/toplist`, `/banner`, `/album` | weapi / eapi | ported, verified live on a China network |
| `/likelist`, `/like` | eapi / weapi | ported, verified live on a China network |
| `/comment/new`, `/comment/like` | eapi / weapi | ported, verified live on a China network |
| `/comment/add` | xeapi + checkToken v3 | will not be implemented (posting comments is out of scope) |

Also ported (as upstream routes): `/register/anonimous`, `/register/xeapikey` (used by the bootstrap).

Not ported (unused by the app): everything else in upstream `module/`.

## Deviations from upstream

- **Startup**: the server binds the port first; xeapi public key / guest token are bootstrapped in the
  background. Device id, xeapi key and guest token are cached in `--data-dir` and reused on the next start,
  so a warm start is ready immediately (requests during a cold first-run bootstrap wait, bounded to 20 s).
- `register_anonimous` keeps the stored device id instead of generating a new one on every call
  (the xeapi `sk` is bound to the device id).
- GET response cache (2 minutes, like upstream's apicache) is more conservative: only an allowlist of read-mostly routes
  is cached, the key includes all params and the login identity (`MUSIC_U`), and responses with `Set-Cookie` or a non-200
  business code are never cached. `--cache-ttl <secs>` / `--no-cache` to change or disable. No CORS middleware, no static
  pages (loopback-only server for the app).
- Not ported: `linuxapi`, `neapi`, `checkToken` (anti-cheat v2/v3, only needed for posting comments, which is out of scope), `unblock`, proxies, random CN IP.
- `/login/qr/check` on upstream failure surfaces as 404 (upstream's catch block throws a ReferenceError).

## Risk control and random Chinese IP

- `randomCNIP=true` (query) or `--random-cn-ip` (server default, like upstream's `ENABLE_RANDOM_CN_IP`) fills
  `X-Real-IP` / `X-Forwarded-For` with a random IP from `data/china_ip_ranges.txt` (taken from api-enhanced). One IP is
  generated per run and reused, like upstream's `global.cnIp`. `randomCNIP=false` turns it off for one request.
  A client address of `::1` also uses it. Not needed on a China network.
- When NetEase answers `-460` (risk control) and no IP was chosen explicitly, the request is retried **once** with a
  random Chinese IP. If it still fails the response gets an extra `neteasyHint` field (Chinese text); `code` / `msg` are
  left untouched.

## Lifecycle (Android / embedding)

`spawn_server(opts)` runs the server on its own thread and returns after the port is bound; `ServerHandle::stop()` stops it
and waits for the thread. JNI: `RustBackend.start(dataDir, port)`, `RustBackend.stop()`, `RustBackend.isRunning()`.

## CI

- `ci.yml`: fmt, clippy (`-D warnings`), tests; plus an `android` job that cross-compiles the `cdylib` for `arm64-v8a` and reports its size.
- `upstream-check.yml` (weekly): regenerates vectors from the latest upstream `crypto.js` and re-runs the crypto tests.
- `smoke.yml` (manual): runs the server against the real NetEase servers and reports each endpoint's result as annotations.

Live A/B check (`compare.yml`) against upstream api-enhanced on the same runner: guest registration, `/song/url/v1`,
`/playlist/track/all`, `/login/status`, `/song/detail` all agree; latency 0-2 s per call.

Run locally: `cargo run -- --port 19800 --data-dir ./data`.

## License

MIT. Crypto constants and request logic are derived from api-enhanced (MIT).
