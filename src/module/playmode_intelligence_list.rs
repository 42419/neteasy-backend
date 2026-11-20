//! 移植自 api-enhanced 的 `module/playmode_intelligence_list.js`。
//! 心动模式：根据当前歌曲和歌单推荐接下来播放的歌曲。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "songId": query.val("id"),
        "type": "fromPlayOne",
        "playlistId": query.val("pid"),
        // 上游：query.sid || query.id
        "startMusicId": query.truthy("sid").or(query.get("id")),
        "count": query.or("count", json!(1)),
    }));
    request.request("/api/playmode/intelligence/list", data, create_option(&query, "")).await
}
