//! 对应 api-enhanced 的 `util/request.js`：按加密方式构造加密请求、发送，
//! 并把响应解密、规整成 [`Answer`]。
//!
//! 支持的加密方式：`weapi`、`eapi`、`api`、`xeapi`（neteasy 应用用到的）。
//! `linuxapi` / `neapi` / `checkToken`（反作弊）有意不移植。

use super::config::*;
use super::crypto::{self, XeapiOptions, XeapiPublicKey};
use super::index::*;
use super::option::Options;
use indexmap::IndexMap;
use rand::RngCore;
use serde_json::{json, Map, Value};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::watch;

/// 上游的模块返回约定：`{ status, body, cookie, redirectUrl? }`。
#[derive(Clone, Debug)]
pub struct Answer {
    pub status: u16,
    pub body: Value,
    pub cookie: Vec<String>,
    pub redirect_url: Option<String>,
}

impl Answer {
    pub fn new(status: u16, body: Value, cookie: Vec<String>) -> Self {
        Answer {
            status,
            body,
            cookie,
            redirect_url: None,
        }
    }
    pub fn error(status: u16, msg: impl Into<String>) -> Self {
        Answer::new(status, json!({ "code": status, "msg": msg.into() }), vec![])
    }
}

pub type AnswerResult = Result<Answer, Answer>;

const SPECIAL_STATUS_CODES: [i64; 8] = [201, 302, 400, 502, 800, 801, 802, 803];

const UA_WEAPI_PC: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0";
const UA_API_ANDROID: &str = "NeteaseMusic/9.5.61.260802021928(9005061);Dalvik/2.1.0 (Linux; U; Android 12; HBN-AL00 Build/cd737a2.0)";
const UA_API_IPHONE: &str = "NeteaseMusic 9.0.90/5038 (iPhone; iOS 16.2; zh_CN)";
const UA_OSX: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

struct OsInfo {
    os: &'static str,
    appver: &'static str,
    osver: &'static str,
    channel: &'static str,
}

fn os_map(name: &str) -> OsInfo {
    match name {
        "linux" => OsInfo {
            os: "linux",
            appver: "1.2.1.0428",
            osver: "Deepin 20.9",
            channel: "netease",
        },
        "android" => OsInfo {
            os: "android",
            appver: "8.20.20.231215173437",
            osver: "14",
            channel: "xiaomi",
        },
        "iphone" => OsInfo {
            os: "iPhone OS",
            appver: "9.0.90",
            osver: "16.2",
            channel: "distribution",
        },
        "osx" => OsInfo {
            os: "osx",
            appver: "3.1.10.5100",
            osver: "15.5",
            channel: "netease",
        },
        _ => OsInfo {
            os: "pc",
            appver: "3.1.17.204416",
            osver: "Microsoft-Windows-10-Professional-build-19045-64bit",
            channel: "netease",
        },
    }
}

/// 可变的运行时状态（上游把这些放在模块级变量和临时目录文件里）。
#[derive(Default)]
struct State {
    device_id: String,
    anonymous_token: String,
    xeapi_key: Option<XeapiPublicKey>,
    xeapi_session_id: String,
    xeapi_session_key: String,
    nmtid: String,
    nmtid_retries_left: u8,
    wnmcid: String,
}

struct Inner {
    http: reqwest::Client,
    data_dir: PathBuf,
    state: RwLock<State>,
    ready: watch::Sender<bool>,
}

/// 请求管线的共享句柄，克隆开销很小。
#[derive(Clone)]
pub struct Request {
    inner: Arc<Inner>,
    wait_ready: bool,
}

/// 构造完成、待发送的请求（与发送分开，便于单元测试）。
#[derive(Debug)]
pub struct Prepared {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub crypto: String,
    pub use_e_r: bool,
    pub use_xeapi: bool,
    /// 是否需要探测 NMTID（eapi 且还没有 NMTID）
    pub nmtid_probe: bool,
}

fn random_hex(bytes: usize) -> String {
    let mut b = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut b);
    hex::encode(b)
}

fn clean_set_cookie(c: &str) -> String {
    // 上游写法：x.replace(/\s*Domain=[^(;|$)]+;*/, '')（去掉 Set-Cookie 里的 Domain 属性）
    let parts: Vec<&str> = c
        .split(';')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty() && !p.starts_with("Domain="))
        .collect();
    parts.join("; ")
}

impl Request {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();
        let _ = std::fs::create_dir_all(&data_dir);
        let read = |name: &str| {
            std::fs::read_to_string(data_dir.join(name))
                .unwrap_or_default()
                .trim()
                .to_string()
        };
        let xeapi_key = serde_json::from_str::<Value>(&read("xeapi_public_key"))
            .ok()
            .and_then(|v| {
                Some(XeapiPublicKey {
                    public_key_b64: v.get("publicKey")?.as_str()?.to_string(),
                    version: v.get("version").map(js_string).unwrap_or_default(),
                    sk: v.get("sk")?.as_str()?.to_string(),
                })
            });
        let device_id = read("device_id");
        let anonymous_token = read("anonymous_token");
        let cached = xeapi_key.is_some() && !device_id.is_empty() && !anonymous_token.is_empty();
        let wnmcid = {
            let letters: String = (0..6)
                .map(|_| (b'a' + (rand::random::<u8>() % 26)) as char)
                .collect();
            format!("{}.{}.01.0", letters, now_ms())
        };
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("http client");
        let (ready, _) = watch::channel(cached);
        Request {
            inner: Arc::new(Inner {
                http,
                data_dir,
                state: RwLock::new(State {
                    device_id,
                    anonymous_token,
                    xeapi_key,
                    nmtid_retries_left: 3,
                    wnmcid,
                    ..Default::default()
                }),
                ready,
            }),
            wait_ready: true,
        }
    }

    /// 引导流程自己使用的句柄（不能等待引导完成，否则会死锁）。
    pub fn for_bootstrap(&self) -> Request {
        Request {
            inner: self.inner.clone(),
            wait_ready: false,
        }
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.inner.http
    }

    pub fn device_id(&self) -> String {
        self.inner.state.read().unwrap().device_id.clone()
    }

    pub fn set_device_id(&self, id: &str) {
        self.inner.state.write().unwrap().device_id = id.to_string();
        self.persist("device_id", id);
    }

    pub fn anonymous_token(&self) -> String {
        self.inner.state.read().unwrap().anonymous_token.clone()
    }

    pub fn set_anonymous_token(&self, token: &str) {
        self.inner.state.write().unwrap().anonymous_token = token.to_string();
        self.persist("anonymous_token", token);
    }

    pub fn xeapi_public_key(&self) -> Option<XeapiPublicKey> {
        self.inner.state.read().unwrap().xeapi_key.clone()
    }

    pub fn set_xeapi_public_key(&self, key: &XeapiPublicKey, device_id: &str) {
        self.inner.state.write().unwrap().xeapi_key = Some(key.clone());
        let v = json!({ "publicKey": key.public_key_b64, "version": key.version, "sk": key.sk, "deviceId": device_id });
        self.persist("xeapi_public_key", &v.to_string());
    }

    pub fn mark_ready(&self) {
        // 用 send_replace：没有订阅者时 `send` 不会更新值
        self.inner.ready.send_replace(true);
    }

    fn persist(&self, name: &str, content: &str) {
        let _ = std::fs::write(self.inner.data_dir.join(name), content);
    }

    async fn wait_until_ready(&self) {
        if !self.wait_ready {
            return;
        }
        let mut rx = self.inner.ready.subscribe();
        if *rx.borrow() {
            return;
        }
        // 首次运行：等待设备 ID、xeapi 公钥和游客 token 就绪，但最多等 20 秒，不会一直挂起
        let _ = tokio::time::timeout(Duration::from_secs(20), rx.wait_for(|v| *v)).await;
    }

    fn process_cookie(&self, cookie: &Cookie, crypto: &str) -> Cookie {
        let st = self.inner.state.read().unwrap();
        let nuid = random_hex(32);
        let os = os_map(cookie.get("os").map(|s| s.as_str()).unwrap_or("pc"));
        let mut c = cookie.clone();
        let pick = |c: &Cookie, k: &str, d: String| {
            c.get(k).filter(|v| !v.is_empty()).cloned().unwrap_or(d)
        };
        c.insert("__remember_me".into(), "true".into());
        c.insert("ntes_kaola_ad".into(), "1".into());
        let nuid_v = pick(cookie, "_ntes_nuid", nuid.clone());
        c.insert("_ntes_nuid".into(), nuid_v);
        c.insert(
            "_ntes_nnid".into(),
            pick(cookie, "_ntes_nnid", format!("{},{}", nuid, now_ms())),
        );
        c.insert("WNMCID".into(), pick(cookie, "WNMCID", st.wnmcid.clone()));
        c.insert("WEVNSM".into(), pick(cookie, "WEVNSM", "1.0.0".into()));
        c.insert("osver".into(), pick(cookie, "osver", os.osver.into()));
        c.insert(
            "deviceId".into(),
            pick(cookie, "deviceId", st.device_id.clone()),
        );
        c.insert("os".into(), pick(cookie, "os", os.os.into()));
        c.insert("channel".into(), pick(cookie, "channel", os.channel.into()));
        c.insert("appver".into(), pick(cookie, "appver", os.appver.into()));
        if let Some(n) = cookie.get("NMTID").filter(|v| !v.is_empty()) {
            c.insert("NMTID".into(), n.clone());
        } else if !st.nmtid.is_empty() {
            c.insert("NMTID".into(), st.nmtid.clone());
        } else if st.nmtid_retries_left == 0 || crypto != "eapi" {
            c.insert("NMTID".into(), format!("00O{}", random_hex(19)));
        }
        if c.get("MUSIC_U").map(|v| v.is_empty()).unwrap_or(true) {
            let cur = c.get("MUSIC_A").filter(|v| !v.is_empty()).cloned();
            c.insert(
                "MUSIC_A".into(),
                cur.unwrap_or_else(|| st.anonymous_token.clone()),
            );
        }
        c
    }

    /// 为 `uri`（如 `/api/song/detail`）构造待发送的请求，对应上游 `createRequest` 里到 `axios(settings)` 之前的部分。
    pub fn prepare(
        &self,
        uri: &str,
        mut data: Map<String, Value>,
        options: &Options,
    ) -> Result<Prepared, Answer> {
        let mut crypto_mode = options.crypto.clone();
        if crypto_mode.is_empty() {
            crypto_mode = if ENCRYPT { "eapi".into() } else { "api".into() };
        }
        // axios 对字符串 body 默认按 form-urlencoded 发送；reqwest 需要显式声明
        let mut headers: Vec<(String, String)> = vec![(
            "Content-Type".into(),
            "application/x-www-form-urlencoded".into(),
        )];
        // 上游：`const ip = options.realIP || options.ip || ''`（ip 是服务器看到的客户端地址）
        let ip = if options.real_ip.is_empty() {
            options.ip.clone()
        } else {
            options.real_ip.clone()
        };
        if !ip.is_empty() {
            headers.push(("X-Real-IP".into(), ip.clone()));
            headers.push(("X-Forwarded-For".into(), ip));
        }
        let cookie = self.process_cookie(&options.cookie, &crypto_mode);
        headers.push(("Cookie".into(), cookie_obj_to_string(&cookie)));
        let csrf = cookie.get("__csrf").cloned().unwrap_or_default();
        let now = now_ms();

        let e_r_opt = options.e_r.clone().or_else(|| data.get("e_r").cloned());
        let e_r = e_r_opt.map(|v| to_boolean(&v)).unwrap_or(ENCRYPT_RESPONSE);
        data.insert("e_r".into(), Value::Bool(e_r));

        let set = |h: &mut Vec<(String, String)>, k: &str, v: String| {
            h.retain(|(n, _)| !n.eq_ignore_ascii_case(k));
            h.push((k.to_string(), v));
        };
        let strip_api = |u: &str| u.get(5..).unwrap_or("").to_string();
        let domain_or = |d: &str| {
            if options.domain.is_empty() {
                d.to_string()
            } else {
                options.domain.clone()
            }
        };
        let ua_or = |d: &str| {
            if options.ua.is_empty() {
                d.to_string()
            } else {
                options.ua.clone()
            }
        };

        let url;
        let mut form: Vec<(String, String)> = Vec::new();
        match crypto_mode.as_str() {
            "weapi" => {
                set(&mut headers, "Referer", domain_or(DOMAIN));
                set(&mut headers, "User-Agent", ua_or(UA_WEAPI_PC));
                data.insert("csrf_token".into(), Value::String(csrf));
                let out = crypto::weapi(&Value::Object(data.clone()).to_string());
                form.push(("params".into(), out.params));
                form.push(("encSecKey".into(), out.enc_sec_key));
                url = format!("{}/weapi/{}", domain_or(DOMAIN), strip_api(uri));
            }
            "xeapi" => {
                let pk = self
                    .xeapi_public_key()
                    .ok_or_else(|| Answer::error(500, "xeapi public key is missing"))?;
                let get = |k: &str| cookie.get(k).cloned().unwrap_or_default();
                let is_android = get("os") == "android";
                let xos = "android".to_string();
                let appver = if is_android && !get("appver").is_empty() {
                    get("appver")
                } else {
                    "9.1.65".into()
                };
                let osver = if is_android && !get("osver").is_empty() {
                    get("osver")
                } else {
                    "16".into()
                };
                let buildver = if !get("buildver").is_empty() {
                    get("buildver")
                } else {
                    now.to_string()[..10].to_string()
                };
                let device_id = get("deviceId");
                let sdevice = if !get("sDeviceId").is_empty() {
                    get("sDeviceId")
                } else {
                    device_id.clone()
                };
                set(&mut headers, "User-Agent", ua_or(UA_API_ANDROID));
                set(&mut headers, "X-Client-Enc-State", "ENCRYPTED".into());
                set(&mut headers, "x-aeapi", "true".into());
                set(
                    &mut headers,
                    "content-type",
                    "application/x-www-form-urlencoded;charset=utf-8".into(),
                );
                set(&mut headers, "x-deviceid", device_id.clone());
                set(&mut headers, "x-os", xos.clone());
                set(&mut headers, "x-osver", osver.clone());
                set(&mut headers, "x-appver", appver.clone());
                set(&mut headers, "x-sdeviceid", sdevice.clone());
                set(&mut headers, "x-buildver", buildver.clone());
                if !get("MUSIC_U").is_empty() {
                    set(&mut headers, "x-music-u", get("MUSIC_U"));
                }
                let mut xc = cookie.clone();
                xc.insert("os".into(), xos.clone());
                xc.insert("osver".into(), osver);
                xc.insert("appver".into(), appver);
                xc.insert("buildver".into(), buildver);
                xc.insert("deviceId".into(), device_id);
                xc.insert("sDeviceId".into(), sdevice);
                set(&mut headers, "Cookie", cookie_obj_to_string(&xc));
                url = format!("{}/xeapi/{}", domain_or(XEAPI_DOMAIN), strip_api(uri));
                let (sid, skey) = {
                    let st = self.inner.state.read().unwrap();
                    (st.xeapi_session_id.clone(), st.xeapi_session_key.clone())
                };
                let body: Vec<(String, String)> = data
                    .iter()
                    .map(|(k, v)| (k.clone(), js_string(v)))
                    .collect();
                let xo = XeapiOptions {
                    os: xos,
                    session_key: if skey.is_empty() {
                        None
                    } else {
                        Some(skey.into_bytes())
                    },
                    session_id: sid,
                    ..Default::default()
                };
                let out = crypto::xeapi(uri, &body, &xo, &pk)
                    .map_err(|e| Answer::error(500, e.to_string()))?;
                form.push(("C".into(), out.c));
                form.push(("S".into(), out.s));
                form.push(("R".into(), out.r));
            }
            "eapi" | "api" => {
                let get = |k: &str| cookie.get(k).cloned().unwrap_or_default();
                let nz = |s: String, d: &str| if s.is_empty() { d.to_string() } else { s };
                let mut header = Map::new();
                header.insert("osver".into(), get("osver").into());
                header.insert("deviceId".into(), get("deviceId").into());
                header.insert("os".into(), get("os").into());
                header.insert("appver".into(), get("appver").into());
                header.insert("versioncode".into(), nz(get("versioncode"), "140").into());
                header.insert("mobilename".into(), get("mobilename").into());
                header.insert(
                    "buildver".into(),
                    nz(get("buildver"), &now.to_string()[..10]).into(),
                );
                header.insert(
                    "resolution".into(),
                    nz(get("resolution"), "1920x1080").into(),
                );
                header.insert("__csrf".into(), csrf.into());
                header.insert("channel".into(), get("channel").into());
                header.insert(
                    "requestId".into(),
                    format!("{}_{:04}", now, rand::random::<u32>() % 1000).into(),
                );
                if !get("MUSIC_U").is_empty() {
                    header.insert("MUSIC_U".into(), get("MUSIC_U").into());
                }
                if !get("MUSIC_A").is_empty() {
                    header.insert("MUSIC_A".into(), get("MUSIC_A").into());
                }
                if crypto_mode == "eapi" && !get("NMTID").is_empty() {
                    header.insert("NMTID".into(), get("NMTID").into());
                }
                let header_cookie = header
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}={}",
                            encode_uri_component(k),
                            encode_uri_component(&js_string(v))
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                set(&mut headers, "Cookie", header_cookie);
                let default_ua = if get("os") == "osx" {
                    UA_OSX
                } else {
                    UA_API_IPHONE
                };
                set(&mut headers, "User-Agent", ua_or(default_ua));
                if crypto_mode == "eapi" {
                    data.insert("header".into(), Value::Object(header));
                    let params = crypto::eapi(uri, &Value::Object(data.clone()).to_string());
                    form.push(("params".into(), params));
                    url = format!("{}/eapi/{}", domain_or(EAPI_DOMAIN), strip_api(uri));
                } else {
                    for (k, v) in &data {
                        form.push((k.clone(), js_string(v)));
                    }
                    url = format!("{}{}", domain_or(API_DOMAIN), uri);
                }
            }
            other => return Err(Answer::error(400, format!("Unknown Crypto: {other}"))),
        }
        let nmtid_probe = {
            let st = self.inner.state.read().unwrap();
            crypto_mode == "eapi"
                && st.nmtid.is_empty()
                && st.nmtid_retries_left > 0
                && options
                    .cookie
                    .get("NMTID")
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
        };
        let body = serde_urlencoded::to_string(&form).unwrap_or_default();
        let use_e_r = (crypto_mode == "eapi" || crypto_mode == "weapi") && e_r;
        Ok(Prepared {
            url,
            headers,
            body,
            use_xeapi: crypto_mode == "xeapi",
            crypto: crypto_mode,
            use_e_r,
            nmtid_probe,
        })
    }

    /// 对应上游的 `request(uri, data, options)`。
    pub async fn request(
        &self,
        uri: &str,
        data: Map<String, Value>,
        options: Options,
    ) -> AnswerResult {
        self.wait_until_ready().await;
        let prepared = self.prepare(uri, data, &options)?;
        let mut rb = self
            .inner
            .http
            .post(&prepared.url)
            .body(prepared.body.clone());
        for (k, v) in &prepared.headers {
            rb = rb.header(k.as_str(), v.as_str());
        }
        if options.timeout_ms > 0 {
            rb = rb.timeout(Duration::from_millis(options.timeout_ms));
        }
        let res = rb
            .send()
            .await
            .map_err(|e| Answer::error(502, e.to_string()))?;
        let res_status = res.status().as_u16();
        let set_cookies: Vec<String> = res
            .headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok().map(|s| s.to_string()))
            .collect();
        let session = (
            res.headers()
                .get("x-encr-ssid")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            res.headers()
                .get("x-encr-sskey")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
        );
        let bytes = res
            .bytes()
            .await
            .map_err(|e| Answer::error(502, e.to_string()))?;

        let mut answer = Answer::new(500, json!({}), vec![]);
        if prepared.nmtid_probe {
            let mut st = self.inner.state.write().unwrap();
            st.nmtid_retries_left = st.nmtid_retries_left.saturating_sub(1);
            for c in &set_cookies {
                if let Some(v) = c.split(';').find_map(|p| p.trim().strip_prefix("NMTID=")) {
                    st.nmtid = v.to_string();
                }
            }
        }
        answer.cookie = set_cookies.iter().map(|c| clean_set_cookie(c)).collect();

        let parsed: Result<Value, String> = if prepared.use_xeapi {
            if let (Some(id), Some(key)) = session {
                let mut st = self.inner.state.write().unwrap();
                st.xeapi_session_id = id;
                st.xeapi_session_key = key;
            }
            crypto::xeapi_res_decrypt(&bytes).map_err(|e| e.to_string())
        } else if prepared.use_e_r {
            crypto::eapi_res_decrypt(&bytes, false).map_err(|e| e.to_string())
        } else {
            serde_json::from_slice::<Value>(&bytes).map_err(|e| e.to_string())
        };
        match parsed {
            Ok(mut body) => {
                if let Some(code) = body.get("code").cloned() {
                    let n = match &code {
                        Value::Number(n) => n.as_i64(),
                        Value::String(s) => s.parse::<i64>().ok(),
                        _ => None,
                    };
                    if let Some(n) = n {
                        body["code"] = json!(n);
                    }
                }
                let code = body.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
                answer.status = if code != 0 { code as u16 } else { res_status };
                if SPECIAL_STATUS_CODES.contains(&code) {
                    answer.status = 200;
                }
                answer.body = body;
            }
            Err(_) => {
                answer.body = Value::String(String::from_utf8_lossy(&bytes).into_owned());
                answer.status = res_status;
            }
        }
        if !(answer.status > 100 && answer.status < 600) {
            answer.status = 400;
        }
        if answer.status == 200 {
            Ok(answer)
        } else {
            log::error!("[ERR] {} -> {} {}", uri, answer.status, answer.body);
            Err(answer)
        }
    }
}

pub type Headers = IndexMap<String, String>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::option::Options;

    fn req() -> Request {
        let dir = std::env::temp_dir().join(format!("neteasy-test-{}", random_hex(4)));
        let r = Request::new(dir);
        r.set_device_id("ABCDEF0123456789");
        r.set_anonymous_token("anon-token");
        r
    }

    fn data(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn weapi_url_and_form() {
        let r = req();
        let p = r
            .prepare(
                "/api/v3/song/detail",
                data(json!({"c":"[{\"id\":1}]"})),
                &Options {
                    crypto: "weapi".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(p.url, "https://music.163.com/weapi/v3/song/detail");
        assert!(p.body.starts_with("params="));
        assert!(p.body.contains("&encSecKey="));
        let cookie = &p.headers.iter().find(|(k, _)| k == "Cookie").unwrap().1;
        assert!(cookie.contains("MUSIC_A=anon-token"));
        assert!(cookie.contains("deviceId=ABCDEF0123456789"));
        assert!(p
            .headers
            .iter()
            .any(|(k, v)| k == "Referer" && v == "https://music.163.com"));
    }

    #[test]
    fn default_crypto_is_eapi_with_header_in_payload() {
        let r = req();
        let p = r
            .prepare(
                "/api/search/get",
                data(json!({"s":"x"})),
                &Options::default(),
            )
            .unwrap();
        assert_eq!(p.url, "https://interfacepc.music.163.com/eapi/search/get");
        assert!(p.body.starts_with("params="));
        assert!(p
            .headers
            .iter()
            .any(|(k, v)| k == "User-Agent" && v.contains("iPhone")));
        assert!(p.nmtid_probe);
    }

    #[test]
    fn xeapi_requires_public_key() {
        let r = req();
        let err = r
            .prepare(
                "/api/song/enhance/player/url/v1",
                data(json!({"ids":"[1]"})),
                &Options {
                    crypto: "xeapi".into(),
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert_eq!(err.status, 500);
    }

    #[test]
    fn xeapi_prepare_sets_headers_and_form() {
        let r = req();
        r.set_xeapi_public_key(
            &XeapiPublicKey {
                public_key_b64: base64_pk(),
                version: "v1".into(),
                sk: "sk".into(),
            },
            "ABCDEF0123456789",
        );
        let p = r
            .prepare(
                "/api/song/enhance/player/url/v1",
                data(json!({"ids":"[1]","level":"exhigh"})),
                &Options {
                    crypto: "xeapi".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(
            p.url,
            "https://interface3.music.163.com/xeapi/song/enhance/player/url/v1"
        );
        assert!(p.body.starts_with("C="));
        assert!(p.body.contains("&S=") && p.body.contains("&R="));
        assert!(p.headers.iter().any(|(k, v)| k == "x-aeapi" && v == "true"));
        assert!(p.headers.iter().any(|(k, v)| k == "x-os" && v == "android"));
    }

    fn base64_pk() -> String {
        use base64::Engine;
        let secret = x25519_dalek::StaticSecret::from([7u8; 32]);
        base64::engine::general_purpose::STANDARD
            .encode(x25519_dalek::PublicKey::from(&secret).as_bytes())
    }

    #[tokio::test]
    async fn mark_ready_releases_waiters_even_without_subscribers() {
        let r = req();
        r.mark_ready();
        // 必须立即返回，而不是等到 20 秒的引导超时
        tokio::time::timeout(Duration::from_millis(500), r.wait_until_ready())
            .await
            .expect("ready");
    }

    #[test]
    fn client_ip_becomes_real_ip_headers() {
        let r = req();
        let o = Options {
            crypto: "weapi".into(),
            ip: "127.0.0.1".into(),
            ..Default::default()
        };
        let p = r.prepare("/api/x", data(json!({})), &o).unwrap();
        assert!(p
            .headers
            .iter()
            .any(|(k, v)| k == "X-Real-IP" && v == "127.0.0.1"));
        assert!(p
            .headers
            .iter()
            .any(|(k, v)| k == "X-Forwarded-For" && v == "127.0.0.1"));
        let o = Options {
            real_ip: "1.2.3.4".into(),
            ..o
        };
        let p = r.prepare("/api/x", data(json!({})), &o).unwrap();
        assert!(p
            .headers
            .iter()
            .any(|(k, v)| k == "X-Real-IP" && v == "1.2.3.4"));
    }

    #[test]
    fn set_cookie_domain_is_stripped() {
        assert_eq!(
            clean_set_cookie("MUSIC_U=abc; Max-Age=100; Domain=.music.163.com; Path=/"),
            "MUSIC_U=abc; Max-Age=100; Path=/"
        );
    }
}
