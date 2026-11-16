//! Port of api-enhanced `module/like.js`.
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let like = query.get("like") != Some("false");
    let data = obj(json!({
        "alg": "itembased",
        "trackId": query.val("id"),
        "like": like,
        "time": "3",
    }));
    request.request("/api/radio/like", data, create_option(&query, "weapi")).await
}
