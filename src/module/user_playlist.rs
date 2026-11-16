//! 移植自 api-enhanced 的 `module/user_playlist.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "uid": query.val("uid"),
        "limit": query.or("limit", json!(30)),
        "offset": query.or("offset", json!(0)),
        "includeVideo": true,
    }));
    request.request("/api/user/playlist", data, create_option(&query, "weapi")).await
}
