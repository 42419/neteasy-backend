//! 移植自 api-enhanced 的 `module/user_detail_new.js`。
//! 用户资料（新版，字段比 `user/detail` 更全）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "all": "true", "userId": query.val("uid") }));
    request.request(&format!("/api/w/v1/user/detail/{}", query.js("uid")), data, create_option(&query, "eapi")).await
}
