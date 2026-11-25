//! 移植自 api-enhanced 的 `module/follow.js`。
//! 关注 / 取消关注用户：`t=1` 关注，其他值取消关注；`id` 是对方的用户 id。
//! 这是写操作，会改动账号的关注列表。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    // 上游：query.t == 1 ? 'follow' : 'delfollow'（宽松相等，"1" 也算）
    let action = if query.get("t") == Some("1") { "follow" } else { "delfollow" };
    request
        .request(
            &format!("/api/user/{action}/{}", query.js("id")),
            obj(json!({})),
            create_option(&query, "weapi"),
        )
        .await
}
