//! 移植自 api-enhanced 的 `module/toplist.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    request.request("/api/toplist", obj(json!({})), create_option(&query, "")).await
}
