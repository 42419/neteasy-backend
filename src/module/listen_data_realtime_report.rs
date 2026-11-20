//! 移植自 api-enhanced 的 `module/listen_data_realtime_report.js`。
//! 实时听歌报告（周 / 月）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "type": query.or("type", json!("week")) }));
    request.request("/api/content/activity/listen/data/realtime/report", data, create_option(&query, "")).await
}
