//! 移植自 api-enhanced 的 `module/comment_event.js`。
//! 动态的评论。`threadId` 是动态的 `threadId`（形如 `A_EV_2_...`）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "limit": query.or("limit", json!(20)),
        "offset": query.or("offset", json!(0)),
        "beforeTime": query.or("before", json!(0)),
    }));
    request
        .request(&format!("/api/v1/resource/comments/{}", query.js("threadId")), data, create_option(&query, "weapi"))
        .await
}
