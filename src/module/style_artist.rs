//! 移植自 api-enhanced 的 `module/style_artist.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "cursor": query.or("cursor", json!(0)),
        "size": query.or("size", json!(20)),
        "tagId": query.val("tagId"),
        "sort": 0,
    }));
    request.request("/api/style-tag/home/artist", data, create_option(&query, "weapi")).await
}
