//! 移植自 api-enhanced 的 `module/recent_listen_list.js`。
//! 最近播放的汇总（歌曲、歌单、专辑、播客混合）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/pc/recent/listen/list", data, create_option(&query, "")).await
}
