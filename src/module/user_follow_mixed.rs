//! 移植自 api-enhanced 的 `module/user_follow_mixed.js`。
//! 关注列表（用户和艺人混合，带翻页游标）。`scene`：0 全部关注，1 关注的歌手，2 关注的用户。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let size = query.or("size", json!(30));
    let cursor = query.or("cursor", json!(0));
    let data = obj(json!({
        "authority": "false",
        // 上游：JSON.stringify({ size, cursor })，是一个 JSON 字符串
        "page": json!({ "size": size, "cursor": cursor }).to_string(),
        "scene": query.or("scene", json!(0)),
        "size": size,
        "sortType": "0",
    }));
    request
        .request("/api/user/follow/users/mixed/get/v2", data, create_option(&query, ""))
        .await
}
