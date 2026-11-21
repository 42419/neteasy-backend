//! 移植自 api-enhanced 的 `module/broadcast_category_region_get.js`。
//! 广播电台的分类和地区列表。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/voice/broadcast/category/region/get", data, create_option(&query, "")).await
}
