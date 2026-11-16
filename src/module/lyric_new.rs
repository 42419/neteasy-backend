//! 移植自 api-enhanced 的 `module/lyric_new.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "id": query.val("id"),
        "cp": false,
        "tv": 0, "lv": 0, "rv": 0, "kv": 0, "yv": 0, "ytv": 0, "yrv": 0,
    }));
    request.request("/api/song/lyric/v1", data, create_option(&query, "")).await
}
