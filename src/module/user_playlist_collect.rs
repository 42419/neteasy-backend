//! 移植自 api-enhanced 的 `module/user_playlist_collect.js`。
//! 收藏的歌单。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "limit": query.or("limit", json!("100")),
        "offset": query.or("offset", json!("0")),
        "userId": query.val("uid"),
        "isWebview": "true",
        "includeRedHeart": "true",
        "includeTop": "true",
    }));
    request.request("/api/user/playlist/collect", data, create_option(&query, "")).await
}
