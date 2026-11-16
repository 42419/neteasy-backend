//! 对应 api-enhanced 的 `util/xeapiKey.js`：xeapi 公钥的获取与解析。

use super::crypto::XeapiPublicKey;
use super::index::{js_string, Query};
use super::request::Request;
use crate::module::register_xeapikey;

/// 对应 `getXeapiPublicKey(currentPublicKey, deviceId)`
pub async fn get_xeapi_public_key(
    request: &Request,
    current: Option<&XeapiPublicKey>,
    device_id: &str,
) -> Result<XeapiPublicKey, String> {
    let mut query = Query::default();
    query
        .params
        .insert("deviceId".into(), device_id.to_string());
    query.params.insert(
        "currentKeyVersion".into(),
        current.map(|c| c.version.clone()).unwrap_or_default(),
    );
    let result = register_xeapikey::handle(query, request)
        .await
        .map_err(|a| a.body.to_string())?;
    let body = result.body;
    let public_key = body
        .get("publicKey")
        .and_then(|v| v.as_str())
        .ok_or("xeapi public key response missing publicKey")?;
    let mut sk = body
        .get("sk")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if sk.is_empty() {
        sk = current.map(|c| c.sk.clone()).unwrap_or_default();
    }
    if sk.is_empty() {
        return Err("xeapi public key response missing sk".into());
    }
    Ok(XeapiPublicKey {
        public_key_b64: public_key.to_string(),
        version: body.get("version").map(js_string).unwrap_or_default(),
        sk,
    })
}
