//! 移植自 api-enhanced 的 `module/voicelist_my_created.js`。
//! 我创建的声音列表。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "limit": query.or("limit", json!(20)) }));
    request.request("/api/social/my/created/voicelist/v1", data, create_option(&query, "weapi")).await
}
