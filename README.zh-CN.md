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
| `/login/qr/key`、`/login/qr/create`、`/login/qr/check`、`/login/status` | eapi / 本地生成 / eapi / weapi | 已移植，国内网络实测通过 |
| `/song/url/v1` | xeapi | 已移植，国内网络实测通过 |
| `/song/detail`、`/lyric`、`/lyric/new` | weapi / eapi | 已移植，国内网络实测通过 |
| `/search`、`/search/suggest`、`/search/hot` | eapi / weapi / eapi | 已移植，国内网络实测通过 |
| `/playlist/detail`、`/playlist/track/all`、`/user/playlist`、`/user/detail` | eapi / weapi | 已移植，国内网络实测通过 |
| `/recommend/songs`、`/personalized`、`/toplist`、`/banner`、`/album` | weapi / eapi | 已移植，国内网络实测通过 |
| `/likelist`、`/like` | eapi / weapi | 已移植，国内网络实测通过 |
| `/comment/new`、`/comment/like` | eapi / weapi | 已移植，国内网络实测通过 |
| `/comment/add` | xeapi + checkToken v3 | 不做（发评论不在范围内，只做获取评论） |

首页用到的接口（给 Flutter 版用）：`homepage/block/page`、`homepage/dragon/ball`、`program/recommend`、
`personalized/newsong`、`personalized/djprogram`、标签歌单、`style/*`、`dj/*`、`top/artists`、`toplist/artist`、
`voicelist/*` 已移植；请求的路径、加密方式和参数由测试解密核对，返回结构用游客身份联网检查过。
详见 [docs/home-api.md](docs/home-api.md)。

另外按上游路由移植了 `/register/anonimous`、`/register/xeapikey`（启动初始化要用）。

未移植（App 没用到）：上游 `module/` 里的其余全部接口。

## 与上游的差异

- **启动**：先绑定端口，xeapi 公钥和游客 token 在后台初始化。设备 ID、xeapi 公钥、游客 token 缓存在
  `--data-dir`，下次启动直接复用，热启动即刻可用（首次冷启动期间的请求会等待初始化，最长 20 秒）。
- `register_anonimous` 沿用已保存的设备 ID，不再每次调用都重新生成（xeapi 的 `sk` 与设备 ID 绑定）。
- GET 响应缓存（2 分钟，对应上游的 apicache）做得更保守：只缓存白名单里读多写少的路由，缓存键包含全部参数和登录身份（`MUSIC_U`），
  带 `Set-Cookie` 或业务码不是 200 的响应一律不缓存。`--cache-ttl <秒>` 调整、`--no-cache` 关闭。没有 CORS 中间件、没有静态页面
  （只在本机回环地址给 App 用）。
- 未移植：`linuxapi`、`neapi`、`checkToken`（反作弊 v2/v3，只有发评论才需要，不在范围内）、`unblock`、代理、随机国内 IP。
- `/login/qr/check` 在上游失败时表现为 404（上游 catch 里引用了作用域外变量，会抛 ReferenceError）。

## 风控与随机中国 IP

- `randomCNIP=true`（query 参数）或 `--random-cn-ip`（服务器默认，对应上游环境变量 `ENABLE_RANDOM_CN_IP`）会把
  `X-Real-IP` / `X-Forwarded-For` 填成 `data/china_ip_ranges.txt`（取自 api-enhanced）里的随机 IP。和上游的 `global.cnIp`
  一样，每次运行只生成一个并一直复用；`randomCNIP=false` 可对单次请求关闭；客户端地址是 `::1` 时同样会用它。国内网络不需要。
- 网易云返回 `-460`（风控）且没有显式指定过 IP 时，会自动换一个随机中国 IP **重试一次**；仍然失败就在响应里多加一个
  中文的 `neteasyHint` 字段，`code` / `msg` 保持原样不动。

## 生命周期（Android / 嵌入使用）

`spawn_server(opts)` 在独立线程里运行服务，端口绑定成功后才返回；`ServerHandle::stop()` 停止服务并等待线程退出。
JNI：`RustBackend.start(dataDir, port)`、`RustBackend.stop()`、`RustBackend.isRunning()`。

## CI

- `ci.yml`：fmt、clippy（`-D warnings`）、测试；另有 `android` 任务，交叉编译 `arm64-v8a` 的 `cdylib` 并报告体积。
- `upstream-check.yml`（每周）：用最新上游 `crypto.js` 重新生成对拍向量并跑加密测试，上游改了加密就会失败。
- `smoke.yml`（手动）：对真实网易云服务器跑一遍各接口，结果以 annotation 形式输出。

在同一台 runner 上与上游 api-enhanced 做的联网 A/B 对照（`compare.yml`）：游客注册、`/song/url/v1`、
`/playlist/track/all`、`/login/status`、`/song/detail` 结果一致，单次调用延迟 0~2 秒。

本地运行：`cargo run -- --port 19800 --data-dir ./data`。

## 许可

MIT。加密常量和请求逻辑派生自 api-enhanced（MIT），详见 `LICENSE` 与 `NOTICE`。
