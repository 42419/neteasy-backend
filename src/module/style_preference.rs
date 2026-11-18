//! 移植自 api-enhanced 的 `module/style_preference.js`。
//! 当前用户的曲风偏好（需要登录）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/tag/my/preference/get", data, create_option(&query, "weapi")).await
}
