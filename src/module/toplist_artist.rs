//! 移植自 api-enhanced 的 `module/toplist_artist.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "type": query.or("type", json!(1)),
        "limit": 100,
        "offset": 0,
        "total": true,
    }));
    request.request("/api/toplist/artist", data, create_option(&query, "weapi")).await
}
