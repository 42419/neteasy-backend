//! 移植自 api-enhanced 的 `module/mlog_mycollect.js`。
//! 收藏的笔记（mlog）。xeapi 加密，测试里无法解密请求体，只核对路径。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "time": query.or("time", json!("-1")),
        "limit": query.or("limit", json!(30)),
        "mlogType": query.or("type", json!(3)),
    }));
    request.request("/api/mlog/mycollect/bytime/get", data, create_option(&query, "xeapi")).await
}
