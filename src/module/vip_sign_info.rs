//! 移植自 api-enhanced 的 `module/vip_sign_info.js`。
//! 会员签到信息（只读，不会签到）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/vipnewcenter/app/user/sign/info", data, create_option(&query, "weapi")).await
}
