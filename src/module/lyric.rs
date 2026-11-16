//! 移植自 api-enhanced 的 `module/lyric.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "id": query.val("id"),
        "tv": -1, "lv": -1, "rv": -1, "kv": -1,
        "_nmclfl": 1,
    }));
    request.request("/api/song/lyric", data, create_option(&query, "")).await
}
