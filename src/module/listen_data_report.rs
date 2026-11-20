//! 移植自 api-enhanced 的 `module/listen_data_report.js`。
//! 听歌报告（周 / 月 / 年）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "type": query.or("type", json!("week")), // week / month / year
        "endTime": query.val("endTime"),         // 不填就是本周/月
    }));
    request.request("/api/content/activity/listen/data/report", data, create_option(&query, "")).await
}
