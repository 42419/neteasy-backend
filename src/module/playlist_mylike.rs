//! 移植自 api-enhanced 的 `module/playlist_mylike.js`。
//! 赞过的歌单 / 视频（「收藏和赞」）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "time": query.or("time", json!("-1")),
        "limit": query.or("limit", json!("12")),
    }));
    request.request("/api/mlog/playlist/mylike/bytime/get", data, create_option(&query, "weapi")).await
}
