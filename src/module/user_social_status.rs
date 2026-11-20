//! 移植自 api-enhanced 的 `module/user_social_status.js`。
//! 用户当前的状态（「添加状态」）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "visitorId": query.val("uid") }));
    request.request("/api/social/user/status", data, create_option(&query, "")).await
}
