//! 移植自 api-enhanced 的 `module/user_event.js`。
//! 某个用户发布的动态（`uid`）。翻页用 `lasttime`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    // 上游用的是 `??`：参数传了（哪怕是空字符串）就用传的值，没传才用默认值
    let data = obj(json!({
        "getcounts": true,
        "time": query.val("lasttime").unwrap_or(json!(-1)),
        "limit": query.val("limit").unwrap_or(json!(30)),
        "total": false,
        "fromRN": "true",
    }));
    request
        .request(&format!("/api/event/get/{}", query.js("uid")), data, create_option(&query, ""))
        .await
}
