//! 移植自 api-enhanced 的 `module/vip_tasks.js`。
//! 会员成长任务列表（只读，不领取）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/vipnewcenter/app/level/task/list", data, create_option(&query, "weapi")).await
}
