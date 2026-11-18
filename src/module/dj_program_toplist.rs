//! 移植自 api-enhanced 的 `module/dj_program_toplist.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "limit": query.or("limit", json!(100)),
        "offset": query.or("offset", json!(0)),
    }));
    request.request("/api/program/toplist/v1", data, create_option(&query, "weapi")).await
}
