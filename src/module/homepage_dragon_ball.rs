//! 移植自 api-enhanced 的 `module/homepage_dragon_ball.js`。
//! 官方首页顶部的圆形入口（每日推荐、歌单、排行榜……）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({}));
    request.request("/api/homepage/dragon/ball/static", data, create_option(&query, "")).await
}
