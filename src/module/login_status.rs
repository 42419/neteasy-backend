//! Port of api-enhanced `module/login_status.js`.
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{Answer, AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let result = request.request("/api/w/nuser/account/get", obj(json!({})), create_option(&query, "weapi")).await?;
    if result.body.get("code").and_then(|c| c.as_i64()) == Some(200) {
        return Ok(Answer::new(200, json!({ "data": result.body }), result.cookie));
    }
    Ok(result)
}
