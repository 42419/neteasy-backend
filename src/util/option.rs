//! 对应 api-enhanced 的 `util/option.js`：单次请求的选项。

use super::index::{Cookie, Query};
use serde_json::Value;

#[derive(Clone, Debug, Default)]
pub struct Options {
    pub crypto: String,
    pub cookie: Cookie,
    pub ua: String,
    pub real_ip: String,
    /// 服务器看到的客户端地址（上游的 `options.ip`）
    pub ip: String,
    /// 是否使用随机中国 IP 填充 X-Real-IP：`None` 表示未指定，沿用服务器的默认设置
    pub random_cn_ip: Option<bool>,
    pub e_r: Option<Value>,
    pub domain: String,
    pub check_token: String,
    pub timeout_ms: u64,
}

/// 对应 `createOption(query, crypto = '', checkToken = false)`
pub fn create_option(query: &Query, crypto: &str) -> Options {
    create_option_with(query, crypto, "")
}

pub fn create_option_with(query: &Query, crypto: &str, check_token: &str) -> Options {
    let crypto = query.truthy("crypto").unwrap_or(crypto).to_string();
    let cookie = if !query.cookie.is_empty() {
        query.cookie.clone()
    } else {
        std::env::var("NETEASE_COOKIE")
            .map(|c| super::index::cookie_to_json(&c))
            .unwrap_or_default()
    };
    Options {
        crypto,
        cookie,
        ua: query.get("ua").unwrap_or("").to_string(),
        real_ip: query.get("realIP").unwrap_or("").to_string(),
        ip: query.ip.clone(),
        // 对应上游的 randomCNIP：query 里写 true/1 开启，false 关闭，不写则沿用服务器默认
        random_cn_ip: query.get("randomCNIP").and_then(|v| match v {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        }),
        e_r: query.truthy("e_r").map(|s| Value::String(s.to_string())),
        domain: query.get("domain").unwrap_or("").to_string(),
        check_token: query
            .truthy("checkToken")
            .unwrap_or(check_token)
            .to_string(),
        timeout_ms: query
            .get("timeout")
            .and_then(|t| t.parse().ok())
            .unwrap_or(0),
    }
}
