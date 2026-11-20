//! 移植自 api-enhanced 的 `module/listen_data_total.js`。
//! 累计听歌数据（累计时长等）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/content/activity/listen/data/total", data, create_option(&query, "")).await
}
