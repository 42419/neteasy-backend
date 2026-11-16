//! Port of api-enhanced `module/register_xeapikey.js`.
use crate::util::crypto;
use crate::util::config::API_DOMAIN;
use crate::util::index::{now_ms, Query};
use crate::util::request::{Answer, AnswerResult, Request};
use serde_json::{json, Value};

fn generate_nonce() -> String {
    (0..16).map(|_| char::from(b'0' + rand::random::<u8>() % 10)).collect()
}

/// Fetch (and verify the signature of) the xeapi public key state for `deviceId`.
pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let nonce = generate_nonce();
    let timestamp = now_ms().to_string();
    let device_id = query.truthy("deviceId").map(|s| s.to_string()).unwrap_or_else(|| request.device_id());
    let current = query.get("currentKeyVersion").unwrap_or("");
    let form = vec![
        ("appVersion", "9.5.61".to_string()),
        ("currentKeyVersion", current.to_string()),
        ("deviceId", device_id.clone()),
        ("nonce", nonce.clone()),
        ("os", "android".to_string()),
        ("requestType", "active".to_string()),
        ("signature", crypto::xeapi_sign(&timestamp, &nonce)),
        ("t1", String::new()),
        ("t2", String::new()),
        ("timestamp", timestamp),
        ("uid", String::new()),
    ];
    let fail = |msg: &str| Answer::error(500, msg.to_string());
    let res = request
        .http()
        .post(format!("{API_DOMAIN}/api/bsr/sk/get"))
        .header(
            "User-Agent",
            "NeteaseMusic/9.5.61.260802021928(9005061);Dalvik/2.1.0 (Linux; U; Android 12; HBN-AL00 Build/cd737a2.0)",
        )
        .header("Cookie", if device_id.is_empty() { String::new() } else { format!("deviceId={}", crate::util::index::encode_uri_component(&device_id)) })
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(serde_urlencoded::to_string(&form).unwrap_or_default())
        .send()
        .await
        .map_err(|e| fail(&e.to_string()))?;
    let bytes = res.bytes().await.map_err(|e| fail(&e.to_string()))?;
    let body: Value = serde_json::from_slice(&bytes).map_err(|e| fail(&e.to_string()))?;
    let data = body.get("data").cloned().unwrap_or(Value::Null);
    let encrypted = data.get("encryptedData").and_then(Value::as_str);
    if body.get("code").and_then(Value::as_i64) != Some(200) || encrypted.is_none() {
        return Err(fail("xeapi public key request failed"));
    }
    let ts = data.get("timestamp").map(crate::util::index::js_string).unwrap_or_default();
    let sig = data.get("signature").and_then(Value::as_str).unwrap_or("");
    if sig.is_empty() || crypto::xeapi_sign(&ts, &nonce) != sig {
        return Err(fail("xeapi public key response signature mismatch"));
    }
    let mut public_key = crypto::xeapi_decrypt_public_key(encrypted.unwrap_or("")).map_err(|e| fail(&e.to_string()))?;
    if public_key.get("sk").and_then(Value::as_str).unwrap_or("").is_empty() {
        return Err(fail("xeapi public key response missing sk"));
    }
    public_key["deviceId"] = json!(device_id);
    Ok(Answer::new(200, public_key, vec![]))
}
