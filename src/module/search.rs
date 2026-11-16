//! 移植自 api-enhanced 的 `module/search.js`。
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
    // type：1 单曲，10 专辑，100 歌手，1000 歌单，1002 用户，1004 MV，1006 歌词，1009 电台，1014 视频
    let data = obj(json!({
        "s": query.val("keywords"),
        "type": query.or("type", json!(1)),
        "limit": query.or("limit", json!(30)),
        "offset": query.or("offset", json!(0)),
    }));
    request.request("/api/search/get", data, create_option(&query, "")).await
}
