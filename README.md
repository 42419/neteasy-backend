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
| `/login/qr/key`, `/login/qr/create`, `/login/qr/check`, `/login/status` | eapi / local / eapi / weapi | todo |
| `/song/url/v1` | xeapi | todo |
| `/song/detail`, `/lyric`, `/lyric/new` | weapi / eapi | todo |
| `/search`, `/search/suggest`, `/search/hot` | eapi / weapi / eapi | todo |
| `/playlist/detail`, `/playlist/track/all`, `/user/playlist`, `/user/detail` | eapi / weapi | todo |
| `/recommend/songs`, `/personalized`, `/toplist`, `/banner`, `/album` | weapi / eapi | todo |
| `/likelist`, `/like` | eapi / weapi | todo |
| `/comment/new`, `/comment/like` | eapi / weapi | todo |
| `/comment/add` | xeapi + checkToken v3 | todo (needs anti-cheat token) |

Not ported (unused by the app): everything else in upstream `module/`.

## License

MIT. Crypto constants and request logic are derived from api-enhanced (MIT).
