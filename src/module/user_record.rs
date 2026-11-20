//! 移植自 api-enhanced 的 `module/user_record.js`。
//! 听歌排行。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "uid": query.val("uid"),
        "type": query.or("type", json!(0)), // 1：最近一周，0：所有时间
    }));
    request.request("/api/v1/play/record", data, create_option(&query, "weapi")).await
}
