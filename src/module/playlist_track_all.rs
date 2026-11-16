//! Port of api-enhanced `module/playlist_track_all.js`.
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{Answer, AnswerResult, Request};
use serde_json::{json, Value};

/// JS `parseInt(x) || default`
fn parse_int_or(v: Option<&str>, default: usize) -> usize {
    v.and_then(|s| s.trim().parse::<usize>().ok()).filter(|n| *n != 0).unwrap_or(default)
}

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "id": query.val("id"),
        "n": 100000,
        "s": query.or("s", json!(8)),
    }));
    let limit = parse_int_or(query.get("limit"), 1000);
    let offset = query.get("offset").and_then(|s| s.trim().parse::<usize>().ok()).unwrap_or(0);
    let res = request.request("/api/v6/playlist/detail", data, create_option(&query, "")).await?;
    // upstream throws a TypeError (no body -> 404) when the playlist is missing
    let track_ids = res
        .body
        .pointer("/playlist/trackIds")
        .and_then(Value::as_array)
        .ok_or_else(|| Answer::new(500, Value::Null, vec![]))?;
    let c = format!(
        "[{}]",
        track_ids
            .iter()
            .skip(offset)
            .take(limit)
            .map(|t| format!("{{\"id\":{}}}", t.get("id").map(crate::util::index::js_string).unwrap_or_default()))
            .collect::<Vec<_>>()
            .join(",")
    );
    request.request("/api/v3/song/detail", obj(json!({ "c": c })), create_option(&query, "")).await
}
