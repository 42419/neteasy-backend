//! 移植自 api-enhanced 的 `module/record_recent_song.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "limit": query.or("limit", json!(100)) }));
    request.request("/api/play-record/song/list", data, create_option(&query, "weapi")).await
}
