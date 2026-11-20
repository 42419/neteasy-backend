//! 移植自 api-enhanced 的 `module/vip_sign_detail.js`。
//! 某天的会员签到详情。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "signDayTime": query.val("timestamp"),
        "type": "1",
    }));
    request.request("/api/vipnewcenter/app/level/user/checkin/history/detail", data, create_option(&query, "eapi")).await
}
