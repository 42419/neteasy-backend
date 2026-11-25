//! 对应 api-enhanced 的 `server.js`：HTTP 路由、把 query/body/cookie 合并成 `Query`、
//! 分发给模块、转发 `Set-Cookie`，以及统一的错误响应格式。
//!
//! 与上游的差异（已记录）：没有静态页面、没有 2 分钟的 GET 响应缓存、没有 CORS 中间件
//!（服务只监听本机回环地址，仅供 App 使用）。

use crate::module::{registry, ModuleFn};
use crate::util::index::{
    cookie_to_json, decode_uri_component, has_cookie, js_string, Cookie, Query,
};
use crate::util::request::{Answer, Request};
use axum::{
    body::{to_bytes, Body},
    extract::{Request as HttpRequest, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::Response,
    Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct App {
    request: Request,
    routes: HashMap<String, ModuleFn>,
    /// 全小写的路由表：和 Express 默认一样，路由匹配不区分大小写（比如 `/digitalAlbum/purchased`）
    routes_ci: HashMap<String, ModuleFn>,
    cache: ResponseCache,
}

/// 默认缓存有效期，对应上游的 `cache('2 minutes')`
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(120);

/// 允许缓存的 GET 路由。比上游的「全部缓存」保守：只放读多写少、过期几分钟无所谓的接口。
/// 登录、点赞、游客注册、喜欢列表、播放地址等有状态或时效性强的接口一律不缓存。
const CACHEABLE_ROUTES: &[&str] = &[
    "/banner",
    "/personalized",
    "/toplist",
    "/search",
    "/search/hot",
    "/search/suggest",
    "/cloudsearch",
    "/search/default",
    "/search/hot/detail",
    "/search/multimatch",
    "/playlist/detail",
    "/playlist/track/all",
    "/album",
    "/song/detail",
    "/lyric",
    "/lyric/new",
    "/recommend/songs",
    "/comment/new",
    "/user/detail",
    // 首页区块和它们点进去的内容：读多写少，缓存两分钟没问题
    "/homepage/block/page",
    "/homepage/dragon/ball",
    "/program/recommend",
    "/personalized/newsong",
    "/personalized/djprogram",
    "/top/playlist",
    "/top/playlist/highquality",
    "/playlist/catlist",
    "/playlist/hot",
    "/style/list",
    "/style/detail",
    "/style/playlist",
    "/style/artist",
    "/style/song",
    "/dj/program/detail",
    "/dj/detail",
    "/dj/program",
    "/dj/program/toplist",
    "/dj/recommend",
    "/top/artists",
    "/toplist/artist",
    "/voicelist/detail",
    "/voicelist/list",
    // 广播电台的公共列表（正在播放的节目 currentinfo 时效性强，不缓存）
    // 话题（公共内容）。动态流 /event 和关注关系是个人数据，不缓存
    "/hot/topic",
    "/topic/detail",
    "/topic/detail/event/hot",
    "/broadcast/category/region/get",
    "/broadcast/channel/list",
];

const CACHE_MAX_ENTRIES: usize = 256;

/// GET 响应缓存（对应上游的 apicache）。
/// 缓存键 = 路由 + 全部参数（含 `timestamp`，带它就等于绕过缓存）+ 登录身份（MUSIC_U），
/// 所以不同账号、游客和登录态之间不会串数据。带 Set-Cookie 或业务码不是 200 的响应不缓存。
pub struct ResponseCache {
    ttl: Duration,
    entries: Mutex<HashMap<String, (Instant, Answer)>>,
}

impl ResponseCache {
    pub fn new(ttl: Duration) -> Self {
        ResponseCache {
            ttl,
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub fn cacheable(&self, path: &str) -> bool {
        !self.ttl.is_zero() && CACHEABLE_ROUTES.contains(&path)
    }

    pub fn key(path: &str, query: &Query) -> String {
        let mut params: Vec<(&String, &String)> = query.params.iter().collect();
        params.sort();
        let params = params
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&");
        let identity = query
            .cookie
            .get("MUSIC_U")
            .map(|s| s.as_str())
            .unwrap_or("");
        format!("{path}?{params}|{identity}")
    }

    pub fn get(&self, key: &str) -> Option<Answer> {
        let mut map = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        match map.get(key) {
            Some((at, answer)) if at.elapsed() < self.ttl => Some(answer.clone()),
            Some(_) => {
                map.remove(key);
                None
            }
            None => None,
        }
    }

    pub fn put(&self, key: String, answer: &Answer) {
        let ok = answer.status == 200
            && answer.cookie.is_empty()
            && answer
                .body
                .get("code")
                .and_then(Value::as_i64)
                .map(|c| c == 200)
                .unwrap_or(true);
        if !ok || self.ttl.is_zero() {
            return;
        }
        let mut map = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        if map.len() >= CACHE_MAX_ENTRIES && !map.contains_key(&key) {
            let ttl = self.ttl;
            map.retain(|_, (at, _)| at.elapsed() < ttl);
            if map.len() >= CACHE_MAX_ENTRIES {
                // 仍然满了：丢掉最旧的一条
                if let Some(oldest) = map
                    .iter()
                    .min_by_key(|(_, (at, _))| *at)
                    .map(|(k, _)| k.clone())
                {
                    map.remove(&oldest);
                }
            }
        }
        map.insert(key, (Instant::now(), answer.clone()));
    }
}

pub fn build_app(request: Request) -> Router {
    build_app_with(request, DEFAULT_CACHE_TTL)
}

/// `cache_ttl` 为 0 表示关闭响应缓存。
pub fn build_app_with(request: Request, cache_ttl: Duration) -> Router {
    let routes: HashMap<String, ModuleFn> = registry()
        .into_iter()
        .map(|(_, route, f)| (route, f))
        .collect();
    let routes_ci = routes.iter().map(|(r, f)| (r.to_lowercase(), *f)).collect();
    Router::new().fallback(handle).with_state(Arc::new(App {
        request,
        routes,
        routes_ci,
        cache: ResponseCache::new(cache_ttl),
    }))
}

fn json_response(status: u16, body: &Value, cookies: &[String]) -> Response {
    let mut res = Response::new(Body::from(body.to_string()));
    *res.status_mut() = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    for c in cookies {
        if let Ok(v) = HeaderValue::from_str(c) {
            res.headers_mut().append(header::SET_COOKIE, v);
        }
    }
    res
}

fn parse_pairs(s: &str) -> Vec<(String, String)> {
    serde_urlencoded::from_str::<Vec<(String, String)>>(s).unwrap_or_default()
}

fn request_cookies(req: &HttpRequest) -> Cookie {
    let mut c = Cookie::new();
    for h in req.headers().get_all(header::COOKIE) {
        if let Ok(s) = h.to_str() {
            for (k, v) in cookie_to_json(s) {
                c.insert(k, decode_uri_component(&v));
            }
        }
    }
    c
}

async fn handle(State(app): State<Arc<App>>, req: HttpRequest) -> Response {
    if req.method() == Method::OPTIONS {
        return Response::builder().status(204).body(Body::empty()).unwrap();
    }
    let path = {
        let p = req.uri().path();
        if p.len() > 1 {
            p.trim_end_matches('/').to_string()
        } else {
            p.to_string()
        }
    };
    if path == "/" {
        return json_response(200, &json!({ "code": 200, "msg": "neteasy-backend" }), &[]);
    }
    let Some(module) = app
        .routes
        .get(&path)
        .or_else(|| app.routes_ci.get(&path.to_lowercase()))
        .copied()
    else {
        return json_response(
            404,
            &json!({ "code": 404, "data": null, "msg": "Not Found" }),
            &[],
        );
    };

    let method = req.method().clone();
    let ip = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|c| c.0.ip().to_string())
        .unwrap_or_default();
    let cookies_from_header = request_cookies(&req);
    let query_str = req.uri().query().unwrap_or("").to_string();
    let is_form = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("application/x-www-form-urlencoded"))
        .unwrap_or(false);
    let is_json = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("application/json"))
        .unwrap_or(false);
    let bytes = to_bytes(req.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap_or_default();
    let body_text = String::from_utf8_lossy(&bytes).into_owned();

    // 对应上游的 Object.assign({}, req.query, req.body, { cookie })
    let mut params = indexmap::IndexMap::<String, String>::new();
    for (k, v) in parse_pairs(&query_str) {
        params.insert(k, v);
    }
    if is_form {
        for (k, v) in parse_pairs(&body_text) {
            params.insert(k, v);
        }
    } else if is_json {
        if let Ok(Value::Object(m)) = serde_json::from_str::<Value>(&body_text) {
            for (k, v) in m {
                params.insert(k, js_string(&v));
            }
        }
    }
    // cookie 的取值顺序：先取 body/query 里的 `cookie` 参数，没有再取请求的 Cookie 头
    let cookie_param = params.shift_remove("cookie");
    let mut cookie = Cookie::new();
    if let Some(c) = cookie_param.filter(|c| !c.is_empty()) {
        cookie = cookie_to_json(&decode_uri_component(&c));
    }
    if !has_cookie(&cookie) {
        cookie = cookies_from_header;
    }
    let no_cookie = params
        .get("noCookie")
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let query = Query { params, cookie, ip };

    // 只缓存白名单里的 GET 请求
    let cache_key = (method == Method::GET && app.cache.cacheable(&path))
        .then(|| ResponseCache::key(&path, &query));
    if let Some(hit) = cache_key.as_deref().and_then(|k| app.cache.get(k)) {
        let mut res = json_response(hit.status, &hit.body, &[]);
        res.headers_mut()
            .insert("x-neteasy-cache", HeaderValue::from_static("hit"));
        return res;
    }

    match module(query, &app.request).await {
        Ok(answer) => {
            if let Some(key) = cache_key {
                app.cache.put(key, &answer);
            }
            let cookies: &[String] = if no_cookie { &[] } else { &answer.cookie };
            json_response(answer.status, &answer.body, cookies)
        }
        Err(answer) => error_response(answer, no_cookie),
    }
}

fn error_response(answer: Answer, no_cookie: bool) -> Response {
    let empty = match &answer.body {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        Value::Object(m) => m.is_empty(),
        _ => false,
    };
    if empty {
        return json_response(
            404,
            &json!({ "code": 404, "data": null, "msg": "Not Found" }),
            &[],
        );
    }
    let mut body = answer.body.clone();
    if body.get("code").and_then(Value::as_i64) == Some(301) {
        body["msg"] = json!("需要登录");
    }
    let cookies: &[String] = if no_cookie { &[] } else { &answer.cookie };
    json_response(answer.status, &body, cookies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;

    fn app() -> Router {
        let dir = std::env::temp_dir().join(format!("neteasy-srv-{}", rand::random::<u32>()));
        build_app(Request::new(dir))
    }

    async fn get(path: &str) -> (u16, Value) {
        let res = app()
            .oneshot(
                axum::http::Request::builder()
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = res.status().as_u16();
        let bytes = to_bytes(res.into_body(), 1 << 20).await.unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    #[tokio::test]
    async fn routes_are_case_insensitive_like_express() {
        // 路由存在时（即便后面联网失败）不应该是路由层的 404 "Not Found"
        let (_, b) = get("/Login/QR/create?key=k").await;
        assert_eq!(b["data"]["qrurl"], "https://music.163.com/login?codekey=k");
    }

    #[tokio::test]
    async fn unknown_route_is_404() {
        let (s, b) = get("/nope").await;
        assert_eq!(s, 404);
        assert_eq!(b["msg"], "Not Found");
    }

    #[tokio::test]
    async fn qr_create_returns_url_and_png_without_network() {
        let (s, b) = get("/login/qr/create?key=abc123&qrimg=true").await;
        assert_eq!(s, 200);
        assert_eq!(
            b["data"]["qrurl"],
            "https://music.163.com/login?codekey=abc123"
        );
        assert!(b["data"]["qrimg"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,"));
    }

    #[tokio::test]
    async fn qr_create_without_qrimg_has_empty_image() {
        let (_, b) = get("/login/qr/create?key=k").await;
        assert_eq!(b["data"]["qrimg"], "");
    }

    fn q(params: &[(&str, &str)], music_u: Option<&str>) -> Query {
        let mut query = Query::default();
        for (k, v) in params {
            query.params.insert(k.to_string(), v.to_string());
        }
        if let Some(u) = music_u {
            query.cookie.insert("MUSIC_U".into(), u.into());
        }
        query
    }

    fn ok_answer() -> Answer {
        Answer::new(200, json!({ "code": 200, "data": [1, 2, 3] }), vec![])
    }

    #[test]
    fn cache_key_separates_accounts_params_and_cache_busting() {
        let guest = ResponseCache::key("/search", &q(&[("keywords", "a")], None));
        let user = ResponseCache::key("/search", &q(&[("keywords", "a")], Some("tok")));
        assert_ne!(guest, user, "游客和登录用户不能共用缓存");
        assert_ne!(
            user,
            ResponseCache::key("/search", &q(&[("keywords", "b")], Some("tok")))
        );
        // 参数顺序无关
        assert_eq!(
            ResponseCache::key("/x", &q(&[("a", "1"), ("b", "2")], None)),
            ResponseCache::key("/x", &q(&[("b", "2"), ("a", "1")], None))
        );
        // 带 timestamp 就等于换了一个键，相当于绕过缓存
        assert_ne!(
            ResponseCache::key("/banner", &q(&[("type", "1")], None)),
            ResponseCache::key("/banner", &q(&[("type", "1"), ("timestamp", "1")], None))
        );
    }

    #[test]
    fn only_read_mostly_routes_are_cacheable() {
        let c = ResponseCache::new(DEFAULT_CACHE_TTL);
        for p in [
            "/banner",
            "/search",
            "/song/detail",
            "/lyric/new",
            "/comment/new",
        ] {
            assert!(c.cacheable(p), "{p}");
        }
        for p in [
            "/login/status",
            "/login/qr/check",
            "/like",
            "/likelist",
            "/comment/like",
            "/song/url/v1",
            "/register/anonimous",
        ] {
            assert!(!c.cacheable(p), "{p} 不应缓存");
        }
        assert!(
            !ResponseCache::new(Duration::ZERO).cacheable("/banner"),
            "ttl=0 即关闭"
        );
    }

    #[test]
    fn cache_hit_expiry_and_rejection_rules() {
        let c = ResponseCache::new(Duration::from_millis(80));
        c.put("k".into(), &ok_answer());
        assert_eq!(c.get("k").unwrap().body["data"], json!([1, 2, 3]));
        std::thread::sleep(Duration::from_millis(120));
        assert!(c.get("k").is_none(), "过期后应当失效");

        c.put(
            "with-cookie".into(),
            &Answer::new(200, json!({ "code": 200 }), vec!["MUSIC_U=x".into()]),
        );
        c.put(
            "biz-error".into(),
            &Answer::new(200, json!({ "code": 801 }), vec![]),
        );
        c.put(
            "http-error".into(),
            &Answer::new(400, json!({ "code": 200 }), vec![]),
        );
        for k in ["with-cookie", "biz-error", "http-error"] {
            assert!(c.get(k).is_none(), "{k} 不应被缓存");
        }
    }

    #[test]
    fn cache_is_bounded() {
        let c = ResponseCache::new(DEFAULT_CACHE_TTL);
        for i in 0..(CACHE_MAX_ENTRIES + 20) {
            c.put(format!("k{i}"), &ok_answer());
        }
        assert!(c.entries.lock().unwrap().len() <= CACHE_MAX_ENTRIES);
        assert!(
            c.get(&format!("k{}", CACHE_MAX_ENTRIES + 19)).is_some(),
            "最新的应当还在"
        );
    }

    #[test]
    fn registry_routes_match_upstream_naming() {
        let routes: Vec<String> = registry().into_iter().map(|(_, r, _)| r).collect();
        for r in [
            "/song/url/v1",
            "/login/qr/check",
            "/playlist/track/all",
            "/lyric/new",
            "/search/suggest",
            "/register/anonimous",
        ] {
            assert!(routes.contains(&r.to_string()), "missing {r}");
        }
    }
}
