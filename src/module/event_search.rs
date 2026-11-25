//! 移植自 api-enhanced 的 `module/event_search.js`。
//! 搜索动态。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    // 上游：query.keyword || query.keywords || ''
    let keyword = query.truthy("keyword").or(query.truthy("keywords")).unwrap_or("");
    let data = obj(json!({
        "keyword": keyword,
        "limit": query.or("limit", json!(30)),
        "offset": query.or("offset", json!(0)),
    }));
    request.request("/api/event/search/list/get/v1", data, create_option(&query, "weapi")).await
}
