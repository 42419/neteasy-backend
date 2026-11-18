//! 移植自 api-enhanced 的 `module/personalized_newsong.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "type": "recommend",
        "limit": query.or("limit", json!(10)),
        "areaId": query.or("areaId", json!(0)),
    }));
    request.request("/api/personalized/newsong", data, create_option(&query, "weapi")).await
}
