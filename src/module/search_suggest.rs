//! 移植自 api-enhanced 的 `module/search_suggest.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "s": query.or("keywords", json!("")) }));
    let kind = if query.get("type") == Some("mobile") { "keyword" } else { "web" };
    request
        .request(&format!("/api/search/suggest/{kind}"), data, create_option(&query, "weapi"))
        .await
}
