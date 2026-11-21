//! 移植自 api-enhanced 的 `module/broadcast_channel_collect_list.js`。
//! 收藏的广播电台。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "contentType": "BROADCAST",
        "limit": query.or("limit", json!("99999")),
        "timeReverseOrder": "true",
        "startDate": "4762584922000",
    }));
    request.request("/api/content/channel/collect/list", data, create_option(&query, "")).await
}
