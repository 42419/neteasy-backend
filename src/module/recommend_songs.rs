//! 移植自 api-enhanced 的 `module/recommend_songs.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "afresh": query.val("afresh") }));
    request.request("/api/v3/discovery/recommend/songs", data, create_option(&query, "weapi")).await
}
