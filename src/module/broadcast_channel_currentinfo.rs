//! 移植自 api-enhanced 的 `module/broadcast_channel_currentinfo.js`。
//! 某个广播电台当前正在播放的节目。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "channelId": query.val("id") }));
    request.request("/api/voice/broadcast/channel/currentinfo", data, create_option(&query, "")).await
}
