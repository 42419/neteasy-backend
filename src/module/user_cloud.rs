//! 移植自 api-enhanced 的 `module/user_cloud.js`。
//! 云盘里的歌曲列表。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "limit": query.or("limit", json!(30)),
        "offset": query.or("offset", json!(0)),
    }));
    request.request("/api/v1/cloud/get", data, create_option(&query, "weapi")).await
}
