//! 移植自 api-enhanced 的 `module/user_account.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/nuser/account/get", data, create_option(&query, "weapi")).await
}
