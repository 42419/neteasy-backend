//! 移植自 api-enhanced 的 `module/vip_info.js`。
//! 会员信息：是否会员、会员类型、到期时间等。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "userId": query.or("uid", json!("")) }));
    request.request("/api/music-vip-membership/front/vip/info", data, create_option(&query, "weapi")).await
}
