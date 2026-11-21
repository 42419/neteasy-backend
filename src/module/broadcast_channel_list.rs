//! 移植自 api-enhanced 的 `module/broadcast_channel_list.js`。
//! 广播电台列表，按分类 / 地区筛选，用 `lastId` 和 `score` 翻页。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "categoryId": query.or("categoryId", json!("0")),
        "regionId": query.or("regionId", json!("0")),
        "limit": query.or("limit", json!("20")),
        "lastId": query.or("lastId", json!("0")),
        "score": query.or("score", json!("-1")),
    }));
    request.request("/api/voice/broadcast/channel/list", data, create_option(&query, "")).await
}
