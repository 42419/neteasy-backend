//! 移植自 api-enhanced 的 `module/vip_sign_history.js`。
//! 会员签到历史。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "type": query.or("type", json!("0")) }));
    request.request("/api/vipnewcenter/app/minidesk/music/sign/pc", data, create_option(&query, "eapi")).await
}
