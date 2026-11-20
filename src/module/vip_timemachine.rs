//! 移植自 api-enhanced 的 `module/vip_timemachine.js`。
//! 黑胶时光机（按周的听歌记录）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let mut data = obj(json!({}));
    // 上游：只有同时传了 startTime 和 endTime 才带上这些字段
    if let (Some(start), Some(end)) = (query.truthy("startTime"), query.truthy("endTime")) {
        data.insert("startTime".into(), json!(start));
        data.insert("endTime".into(), json!(end));
        data.insert("type".into(), json!(1));
        data.insert("limit".into(), query.or("limit", json!(60)));
    }
    request.request("/api/vipmusic/newrecord/weekflow", data, create_option(&query, "weapi")).await
}
