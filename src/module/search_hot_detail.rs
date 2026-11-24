//! 移植自 api-enhanced 的 `module/search_hot_detail.js`。
//! 热搜榜（带副标题、热度、图标，比 `search/hot` 信息多）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/hotsearchlist/get", data, create_option(&query, "weapi")).await
}
