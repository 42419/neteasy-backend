//! 对应 api-enhanced 的 `util/index.js`：cookie 工具、设备 ID 等通用小工具。

use indexmap::IndexMap;
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use rand::Rng;
use serde_json::{Map, Value};

/// 有序的 cookie 映射（和 JS 对象一样保留插入顺序）。
pub type Cookie = IndexMap<String, String>;

/// JS 的 `encodeURIComponent` 不转义 `A-Za-z0-9 - _ . ! ~ * ' ( )`。
const URI_COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

pub fn encode_uri_component(s: &str) -> String {
    utf8_percent_encode(s, URI_COMPONENT).to_string()
}

/// 对应 `safe-decode-uri-component`：解码，但永不失败（非法序列原样保留）。
pub fn decode_uri_component(s: &str) -> String {
    percent_decode_str(s).decode_utf8_lossy().into_owned()
}

/// 对应 `toBoolean`
pub fn to_boolean(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::String(s) => s == "true" || s == "1",
        Value::Number(n) => n.as_f64() == Some(1.0),
        _ => false,
    }
}

/// 对应 `cookieToJson`：只按第一个 `=` 切分，值里的 `=` 原样保留。
pub fn cookie_to_json(cookie: &str) -> Cookie {
    let mut obj = Cookie::new();
    for item in cookie.split(';') {
        if let Some(sep) = item.find('=') {
            if sep > 0 {
                obj.insert(
                    item[..sep].trim().to_string(),
                    item[sep + 1..].trim().to_string(),
                );
            }
        }
    }
    obj
}

/// 对应 `hasCookie`
pub fn has_cookie(c: &Cookie) -> bool {
    !c.is_empty()
}

/// 对应 `cookieObjToString`
pub fn cookie_obj_to_string(c: &Cookie) -> String {
    c.iter()
        .map(|(k, v)| format!("{}={}", encode_uri_component(k), encode_uri_component(v)))
        .collect::<Vec<_>>()
        .join("; ")
}

/// 对应 `generateDeviceId`：52 位大写十六进制字符。
pub fn generate_device_id() -> String {
    const HEX: &[u8] = b"0123456789ABCDEF";
    let mut rng = rand::thread_rng();
    (0..52).map(|_| HEX[rng.gen_range(0..16)] as char).collect()
}

/// 对应 `generateChainId`
pub fn generate_chain_id(cookie: &Cookie) -> String {
    let device = cookie
        .get("sDeviceId")
        .cloned()
        .unwrap_or_else(|| format!("unknown-{}", rand::thread_rng().gen_range(0..1_000_000)));
    format!("v1_{}_web_login_{}", device, now_ms())
}

pub fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// 等价于 JS 的 `String(value)`，用于请求体里出现的标量值。
pub fn js_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".into(),
        other => other.to_string(),
    }
}

pub type Data = Map<String, Value>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_roundtrip_keeps_equals_in_value() {
        let c = cookie_to_json("MUSIC_U=abc==; __csrf=xyz; bad;=nope");
        assert_eq!(c.get("MUSIC_U").unwrap(), "abc==");
        assert_eq!(c.get("__csrf").unwrap(), "xyz");
        assert_eq!(c.len(), 2);
        assert_eq!(cookie_obj_to_string(&c), "MUSIC_U=abc%3D%3D; __csrf=xyz");
    }

    #[test]
    fn encode_matches_js_encode_uri_component() {
        assert_eq!(
            encode_uri_component("a b/中!*'()~-_."),
            "a%20b%2F%E4%B8%AD!*'()~-_."
        );
    }

    #[test]
    fn device_id_shape() {
        let d = generate_device_id();
        assert_eq!(d.len(), 52);
        assert!(d
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase()));
    }

    #[test]
    fn to_boolean_semantics() {
        assert!(to_boolean(&Value::String("true".into())));
        assert!(to_boolean(&Value::String("1".into())));
        assert!(!to_boolean(&Value::String("false".into())));
    }
}

/// 传给每个模块的 `query` 对象（上游：合并后的 req.query + req.body + 解析后的 cookie）。
/// 请求参数都是字符串，和 Express 的 query 对象一致。
#[derive(Clone, Debug, Default)]
pub struct Query {
    pub params: IndexMap<String, String>,
    pub cookie: Cookie,
    /// HTTP 客户端的对端地址（上游的 `req.ip`）
    pub ip: String,
}

impl Query {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    /// 字符串参数的 JS 真值判断：存在且非空。
    pub fn truthy(&self, key: &str) -> Option<&str> {
        self.get(key).filter(|s| !s.is_empty())
    }

    /// 把 `query.key` 当作 JSON 字符串值返回；对应 JS 里的 `undefined` 时返回 `None`。
    pub fn val(&self, key: &str) -> Option<Value> {
        self.get(key).map(|s| Value::String(s.to_string()))
    }

    /// 对应 `query.key || default`
    pub fn or(&self, key: &str, default: Value) -> Value {
        self.truthy(key)
            .map(|s| Value::String(s.to_string()))
            .unwrap_or(default)
    }

    /// 用于字符串模板的 `query.key`（缺失时和 JS 一样得到 "undefined"）。
    pub fn js(&self, key: &str) -> String {
        self.get(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "undefined".into())
    }
}

/// 由 `json!({...})` 字面量构造请求的 `data` 对象（保留插入顺序）。
pub fn obj(v: Value) -> Data {
    match v {
        Value::Object(m) => m,
        _ => Data::new(),
    }
}
