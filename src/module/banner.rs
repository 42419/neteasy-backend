//! 移植自 api-enhanced 的 `module/banner.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let client_type = match query.get("type").unwrap_or("0") {
        "1" => "android",
        "2" => "iphone",
        "3" => "ipad",
        _ => "pc",
    };
    request
        .request("/api/v2/banner/get", obj(json!({ "clientType": client_type })), create_option(&query, ""))
        .await
}
