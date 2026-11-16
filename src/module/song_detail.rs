//! 移植自 api-enhanced 的 `module/song_detail.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let ids: Vec<String> = query
        .get("ids")
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let c = format!(
        "[{}]",
        ids.iter().map(|id| format!("{{\"id\":{}}}", id)).collect::<Vec<_>>().join(",")
    );
    request.request("/api/v3/song/detail", obj(json!({ "c": c })), create_option(&query, "weapi")).await
}
