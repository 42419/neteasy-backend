//! 移植自 api-enhanced 的 `module/vip_tasks_v1.js`。
//! 会员任务进度（xeapi 加密，测试里只核对路径）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "taskType": "app_vip_task_center",
        "userId": query.val("id"),
    }));
    request.request("/api/middle/vip/mission/user/progress/list", data, create_option(&query, "xeapi")).await
}
