//! 移植自 api-enhanced 的 `module/register_anonimous.js`。
use crate::util::index::{generate_device_id, obj, Query};
use crate::util::option::create_option;
use crate::util::request::{Answer, AnswerResult, Request};
use base64::{engine::general_purpose::STANDARD, Engine};
use md5::{Digest, Md5};
use serde_json::{json, Value};

const ID_XOR_KEY_1: &[u8] = b"3go8&$8*3*3h0k(2)2";

fn cloudmusic_dll_encode_id(id: &str) -> String {
    let xored: Vec<u8> = id.bytes().enumerate().map(|(i, b)| b ^ ID_XOR_KEY_1[i % ID_XOR_KEY_1.len()]).collect();
    STANDARD.encode(Md5::digest(&xored))
}

/// 游客 cookie。与上游的差异：只有在还没保存设备 ID 时才生成新的
///（上游每次启动都重新生成），这样设备 ID、xeapi 公钥和 token 在多次运行间保持一致。
pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let mut device_id = request.device_id();
    if device_id.is_empty() {
        device_id = generate_device_id();
        request.set_device_id(&device_id);
    }
    let encoded = STANDARD.encode(format!("{} {}", device_id, cloudmusic_dll_encode_id(&device_id)));
    let data = obj(json!({ "username": encoded }));
    let result = request.request("/api/register/anonimous", data, create_option(&query, "xeapi")).await?;
    if result.body.get("code").and_then(Value::as_i64) == Some(200) {
        let mut body = result.body.clone();
        body["cookie"] = Value::String(result.cookie.join(";"));
        return Ok(Answer::new(200, body, result.cookie));
    }
    Ok(result)
}
