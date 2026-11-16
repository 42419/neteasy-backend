//! Port of api-enhanced `module/search_hot.js`.
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    request.request("/api/search/hot", obj(json!({ "type": 1111 })), create_option(&query, "")).await
}
