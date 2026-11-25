//! 移植自 api-enhanced 的 `module/resource_like.js`。
//! 给资源点赞 / 取消点赞（动态、MV、歌单等）：`t=1` 点赞，其他值取消。
//! `type` 是资源类型（见 `util/config.rs` 的 `resource_type`），`id` 是资源 id；
//! 点赞动态时 `type=6`，并且要传动态自己的 `threadId`。这是写操作。
use crate::util::config::resource_type;
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let t = if query.get("t") == Some("1") { "like" } else { "unlike" };
    let rtype = query.get("type").and_then(resource_type);
    let mut data = obj(json!({
        "threadId": format!("{}{}", rtype.unwrap_or("undefined"), query.js("id")),
    }));
    if rtype == Some("A_EV_2_") {
        // 动态：用请求里的 threadId；没传时和 JS 的 undefined 一样，字段不出现
        match query.val("threadId") {
            Some(v) => data.insert("threadId".into(), v),
            None => data.remove("threadId"),
        };
    }
    request.request(&format!("/api/resource/{t}"), data, create_option(&query, "weapi")).await
}
