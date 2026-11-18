//! 移植自 api-enhanced 的 `module/dj_recommend.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/djradio/recommend/v1", data, create_option(&query, "weapi")).await
}
