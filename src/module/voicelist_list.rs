//! 移植自 api-enhanced 的 `module/voicelist_list.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "limit": query.or("limit", json!("200")),
        "offset": query.or("offset", json!("0")),
        "voiceListId": query.val("voiceListId"),
    }));
    request.request("/api/voice/workbench/voices/by/voicelist", data, create_option(&query, "")).await
}
