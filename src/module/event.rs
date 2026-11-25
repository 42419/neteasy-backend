//! 移植自 api-enhanced 的 `module/event.js`。
//! 关注的人的动态流（「关注」页的主体）。翻页时把上一页返回的 `lasttime` 带回来。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "pagesize": query.or("pagesize", json!(20)),
        "lasttime": query.or("lasttime", json!(-1)), // 翻页：上一页返回的 lasttime
    }));
    request.request("/api/v1/event/get", data, create_option(&query, "weapi")).await
}
