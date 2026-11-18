//! 移植自 api-enhanced 的 `module/top_playlist_highquality.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "cat": query.or("cat", json!("全部")),
        "limit": query.or("limit", json!(50)),
        "lasttime": query.or("before", json!(0)), // 上一页最后一个歌单的 updateTime
        "total": true,
    }));
    request.request("/api/playlist/highquality/list", data, create_option(&query, "weapi")).await
}
