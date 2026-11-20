//! 移植自 api-enhanced 的 `module/signin_progress.js`。
//! 签到进度（只读，不会签到）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "moduleId": query.or("moduleId", json!("1207signin-1207signin")) }));
    request.request("/api/act/modules/signin/v2/progress", data, create_option(&query, "weapi")).await
}
