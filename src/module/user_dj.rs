//! 移植自 api-enhanced 的 `module/user_dj.js`。
//! 用户创建的播客节目。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "limit": query.or("limit", json!(30)),
        "offset": query.or("offset", json!(0)),
    }));
    request.request(&format!("/api/dj/program/{}", query.js("uid")), data, create_option(&query, "weapi")).await
}
