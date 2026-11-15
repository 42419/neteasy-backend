# neteasy-backend

[English](README.md) | 简体中文

[neteasy](https://github.com/42419/neteasy) Android 应用所用接口的 Rust 移植版，上游为
[NeteaseCloudMusicApiEnhanced](https://github.com/NeteaseCloudMusicApiEnhanced/api-enhanced)（下称 api-enhanced）。
对外仍然是 `127.0.0.1:19800` 上的同一套 HTTP 接口，所以 App 里的 `ApiClient` 无需改动；
用它替换内嵌的 `nodejs-mobile` 后端，不再有 JS 运行时，也没有模块加载带来的启动开销。

## 目录结构（与 api-enhanced 一一对应）

| 上游                   | 本项目                    | 职责                                                  |
|------------------------|---------------------------|-------------------------------------------------------|
| `util/crypto.js`       | `src/util/crypto.rs`      | weapi / eapi / xeapi 的请求与响应加解密               |
| `util/request.js`      | `src/util/request.rs`     | 请求管线：cookie、请求头、按加密方式分发              |
| `util/option.js`       | `src/util/option.rs`      | 单次请求的选项                                        |
| `util/config.json`     | `src/util/config.rs`      | 域名、应用常量                                        |
| `util/xeapiKey.js`     | `src/util/xeapi_key.rs`   | xeapi 公钥与会话状态                                  |
| `module/*.js`          | `src/module/*.rs`         | 一个接口一个文件；`song_url_v1` 对应 `/song/url/v1`   |
| `server.js`            | `src/server.rs`           | HTTP 路由、query/cookie 处理、Set-Cookie              |
| `generateConfig.js`    | `src/generate_config.rs`  | 匿名 token 与 xeapi 公钥的启动初始化                  |

对照的上游版本：api-enhanced 4.41.1（`75bf2e5`）。

维护约定：新增或修改接口时，先找到上游对应的文件，保持文件名、函数职责和参数命名一致，
这样上游更新时可以按文件逐个对照移植。

## 校验方式

`scripts/gen_vectors.js` 会在注入固定随机数的前提下运行上游的 `util/crypto.js`，
生成 `tests/vectors.json`；`tests/crypto_vectors.rs` 断言 Rust 的输出与之逐字节一致
（覆盖 weapi、eapi、xeapi（含会话密钥路径）、响应解密、签名、公钥解密）。

```bash
cargo test
```

上游更新加密逻辑后，重新生成向量即可发现不一致：

```bash
node scripts/gen_vectors.js <api-enhanced>/util/crypto.js
cargo test
```

（`crypto.js` 依赖 `crypto-js` 和 `node-forge`，并会 `require('./zstd')`，
在临时目录里安装这两个包、把 `zstd` 桩成空模块即可。）

## 移植状态

App 实际调用的接口（取自 `ApiClient` 的用法），以及上游使用的加密方式：

| 路由 | 加密方式 | 状态 |
|------|----------|------|
| `/login/qr/key`、`/login/qr/create`、`/login/qr/check`、`/login/status` | eapi / 本地生成 / eapi / weapi | 待做 |
| `/song/url/v1` | xeapi | 待做 |
| `/song/detail`、`/lyric`、`/lyric/new` | weapi / eapi | 待做 |
| `/search`、`/search/suggest`、`/search/hot` | eapi / weapi / eapi | 待做 |
| `/playlist/detail`、`/playlist/track/all`、`/user/playlist`、`/user/detail` | eapi / weapi | 待做 |
| `/recommend/songs`、`/personalized`、`/toplist`、`/banner`、`/album` | weapi / eapi | 待做 |
| `/likelist`、`/like` | eapi / weapi | 待做 |
| `/comment/new`、`/comment/like` | eapi / weapi | 待做 |
| `/comment/add` | xeapi + checkToken v3 | 待做（需要反作弊 token） |

已完成：加密核心（`util/crypto.rs`）。

未移植（App 没用到）：上游 `module/` 里的其余全部接口。

## 许可

MIT。加密常量和请求逻辑派生自 api-enhanced（MIT），详见 `LICENSE` 与 `NOTICE`。
