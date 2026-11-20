//! 移植自 api-enhanced 的 `module/user_followeds.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "userId": query.val("uid"),
        "time": "0",
        "limit": query.or("limit", json!(20)),
        "offset": query.or("offset", json!(0)),
        "getcounts": "true",
    }));
    request.request(&format!("/api/user/getfolloweds/{}", query.js("uid")), data, create_option(&query, "")).await
}
