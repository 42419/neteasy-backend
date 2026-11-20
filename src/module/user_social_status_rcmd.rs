//! 移植自 api-enhanced 的 `module/user_social_status_rcmd.js`。
//! 推荐的状态列表。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/social/user/status/rcmd", data, create_option(&query, "")).await
}
