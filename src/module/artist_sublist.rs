//! 移植自 api-enhanced 的 `module/artist_sublist.js`。
//! 收藏的艺人。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "limit": query.or("limit", json!(25)),
        "offset": query.or("offset", json!(0)),
        "total": true,
    }));
    request.request("/api/artist/sublist", data, create_option(&query, "weapi")).await
}
