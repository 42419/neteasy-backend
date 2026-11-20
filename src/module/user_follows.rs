//! 移植自 api-enhanced 的 `module/user_follows.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "offset": query.or("offset", json!(0)),
        "limit": query.or("limit", json!(30)),
        "order": true,
    }));
    request.request(&format!("/api/user/getfollows/{}", query.js("uid")), data, create_option(&query, "weapi")).await
}
