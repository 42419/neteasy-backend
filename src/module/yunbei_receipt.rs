//! 移植自 api-enhanced 的 `module/yunbei_receipt.js`。
//! 云贝收入记录。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "limit": query.or("limit", json!(10)),
        "offset": query.or("offset", json!(0)),
    }));
    request.request("/api/point/receipt", data, create_option(&query, "")).await
}
