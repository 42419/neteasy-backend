//! 移植自 api-enhanced 的 `module/yunbei_tasks.js`。
//! 云贝任务列表（只读，不会签到或领取）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/usertool/task/list/all", data, create_option(&query, "weapi")).await
}
