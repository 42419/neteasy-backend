//! 移植自 api-enhanced 的 `module/vip_growthpoint_details.js`。
//! 会员成长值明细。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "limit": query.or("limit", json!(20)),
        "offset": query.or("offset", json!(0)),
    }));
    request.request("/api/vipnewcenter/app/level/growth/details", data, create_option(&query, "weapi")).await
}
