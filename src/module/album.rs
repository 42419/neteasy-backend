//! 移植自 api-enhanced 的 `module/album.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    request
        .request(&format!("/api/v1/album/{}", query.js("id")), obj(json!({})), create_option(&query, "weapi"))
        .await
}
