//! Port of api-enhanced `module/playlist_detail.js`.
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "id": query.val("id"),
        "n": 100000,
        "s": query.or("s", json!(8)),
    }));
    request.request("/api/v6/playlist/detail", data, create_option(&query, "")).await
}
