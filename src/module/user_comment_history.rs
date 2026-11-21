//! 移植自 api-enhanced 的 `module/user_comment_history.js`。
//! 用户发过的评论（「我的评论」）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "compose_reminder": "true",
        "compose_hot_comment": "true",
        "limit": query.or("limit", json!(10)),
        "user_id": query.val("uid"),
        "time": query.or("time", json!(0)), // 翻页：上一页最后一条评论的时间
    }));
    request.request("/api/comment/user/comment/history", data, create_option(&query, "weapi")).await
}
