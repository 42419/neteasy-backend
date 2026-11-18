//! 移植自 api-enhanced 的 `module/dj_program.js`。
use crate::util::index::{obj, to_boolean, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::{json, Value};

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "radioId": query.val("rid"),
        "limit": query.or("limit", json!(30)),
        "offset": query.or("offset", json!(0)),
        "asc": query.get("asc").map(|s| to_boolean(&Value::String(s.to_string()))).unwrap_or(false),
    }));
    request.request("/api/dj/program/byradio", data, create_option(&query, "weapi")).await
}
