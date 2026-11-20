//! 移植自 api-enhanced 的 `module/listen_data_song_play_rank.js`。
//! 听歌歌曲排行（周 / 月）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "type": query.or("type", json!("month")), // week / month
        "endTime": query.val("endTime"),
    }));
    request.request("/api/content/activity/listen/data/song/play/rank", data, create_option(&query, "")).await
}
