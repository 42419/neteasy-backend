//! 移植自 api-enhanced 的 `module/user_subcount.js`。
//! 各类收藏数量：创建/收藏的歌单、收藏的专辑/艺人/MV/播客等。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/subcount", data, create_option(&query, "weapi")).await
}
