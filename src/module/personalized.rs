//! 移植自 api-enhanced 的 `module/personalized.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "limit": query.or("limit", json!(30)),
        "total": true,
        "n": 1000,
    }));
    request.request("/api/personalized/playlist", data, create_option(&query, "weapi")).await
}
