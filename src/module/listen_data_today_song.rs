//! 移植自 api-enhanced 的 `module/listen_data_today_song.js`。
//! 今日听歌排行。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/content/activity/listen/data/today/song/play/rank", data, create_option(&query, "")).await
}
