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
use std::sync::Arc;

struct App {
    request: Request,
    routes: HashMap<String, ModuleFn>,
}

pub fn build_app(request: Request) -> Router {
    let routes = registry()
        .into_iter()
        .map(|(_, route, f)| (route, f))
        .collect();
    Router::new()
        .fallback(handle)
        .with_state(Arc::new(App { request, routes }))
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
    let Some(module) = app.routes.get(&path).copied() else {
        return json_response(
            404,
            &json!({ "code": 404, "data": null, "msg": "Not Found" }),
            &[],
        );
    };

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

    match module(query, &app.request).await {
        Ok(answer) => {
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
