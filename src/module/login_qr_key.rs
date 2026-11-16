//! Port of api-enhanced `module/login_qr_key.js`.
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{Answer, AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "type": 3 }));
    let result = request.request("/api/login/qrcode/unikey", data, create_option(&query, "")).await?;
    Ok(Answer::new(200, json!({ "data": result.body, "code": 200 }), result.cookie))
}
