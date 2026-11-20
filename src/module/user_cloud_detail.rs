//! 移植自 api-enhanced 的 `module/user_cloud_detail.js`。
//! 云盘歌曲详情，`id` 用英文逗号分隔多个。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    // 上游：query.id.replace(/\s/g, '').split(',')
    let compact: String = query.get("id").unwrap_or("").chars().filter(|c| !c.is_whitespace()).collect();
    let ids: Vec<&str> = compact.split(',').collect();
    let data = obj(json!({ "songIds": ids }));
    request.request("/api/v1/cloud/get/byids", data, create_option(&query, "weapi")).await
}
