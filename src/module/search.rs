//! Port of api-enhanced `module/search.js`.
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    if query.get("type") == Some("2000") {
        let data = obj(json!({
            "keyword": query.val("keywords"),
            "scene": "normal",
            "limit": query.or("limit", json!(30)),
            "offset": query.or("offset", json!(0)),
        }));
        return request.request("/api/search/voice/get", data, create_option(&query, "")).await;
    }
    // type: 1 single, 10 album, 100 artist, 1000 playlist, 1002 user, 1004 MV, 1006 lyric, 1009 radio, 1014 video
    let data = obj(json!({
        "s": query.val("keywords"),
        "type": query.or("type", json!(1)),
        "limit": query.or("limit", json!(30)),
        "offset": query.or("offset", json!(0)),
    }));
    request.request("/api/search/get", data, create_option(&query, "")).await
}
