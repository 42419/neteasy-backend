//! 移植自 api-enhanced 的 `module/user_medal.js`。
//! 用户徽章（「N 枚徽章」）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "uid": query.val("uid") }));
    request.request("/api/medal/user/page", data, create_option(&query, "")).await
}
