//! 移植自 api-enhanced 的 `module/user_mutualfollow_get.js`。
//! 和某个用户的互相关注关系（「我的好友」）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "friendid": query.val("uid") }));
    request.request("/api/user/mutualfollow/get", data, create_option(&query, "")).await
}
