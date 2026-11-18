//! 移植自 api-enhanced 的 `module/playlist_hot.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/playlist/hottags", data, create_option(&query, "weapi")).await
}
