//! 移植自 api-enhanced 的 `module/program_recommend.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "cateId": query.val("type"),
        "limit": query.or("limit", json!(10)),
        "offset": query.or("offset", json!(0)),
    }));
    request.request("/api/program/recommend/v1", data, create_option(&query, "weapi")).await
}
