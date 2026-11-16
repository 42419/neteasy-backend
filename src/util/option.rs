//! Mirrors api-enhanced `util/option.js`.

use super::index::{Cookie, Query};
use serde_json::Value;

#[derive(Clone, Debug, Default)]
pub struct Options {
    pub crypto: String,
    pub cookie: Cookie,
    pub ua: String,
    pub real_ip: String,
    /// client address as seen by the server (upstream `options.ip`)
    pub ip: String,
    pub e_r: Option<Value>,
    pub domain: String,
    pub check_token: String,
    pub timeout_ms: u64,
}

/// `createOption(query, crypto = '', checkToken = false)`
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
