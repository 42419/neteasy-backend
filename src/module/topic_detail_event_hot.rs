//! 移植自 api-enhanced 的 `module/topic_detail_event_hot.js`。
//! 话题下的热门动态。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "actid": query.val("actid") }));
    request.request("/api/act/event/hot", data, create_option(&query, "weapi")).await
}
