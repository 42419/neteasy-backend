//! 移植自 api-enhanced 的 `module/yunbei_task_list_v1.js`。
//! 云贝任务分发列表（只读，不会签到或领取）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/ad/power/yunbei/distribution/list", data, create_option(&query, "weapi")).await
}
