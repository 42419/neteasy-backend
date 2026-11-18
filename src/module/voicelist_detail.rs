//! 移植自 api-enhanced 的 `module/voicelist_detail.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "id": query.val("id") }));
    request.request("/api/voice/workbench/voicelist/detail", data, create_option(&query, "")).await
}
