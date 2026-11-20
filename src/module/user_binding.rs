//! 移植自 api-enhanced 的 `module/user_binding.js`。
//! 账号绑定信息（手机、第三方账号）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request(&format!("/api/v1/user/bindings/{}", query.js("uid")), data, create_option(&query, "weapi")).await
}
