//! 移植自 api-enhanced 的 `module/search_default.js`。
//! 搜索框里的默认搜索词。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/search/defaultkeyword/get", data, create_option(&query, "")).await
}
