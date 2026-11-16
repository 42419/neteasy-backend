//! 移植自 api-enhanced 的 `module/likelist.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    request
        .request("/api/song/like/get", obj(json!({ "uid": query.val("uid") })), create_option(&query, ""))
        .await
}
