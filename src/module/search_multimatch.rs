//! 移植自 api-enhanced 的 `module/search_multimatch.js`。
//! 搜索综合匹配（命中的歌手、专辑、歌单等卡片）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "type": query.or("type", json!(1)),
        "s": query.or("keywords", json!("")),
    }));
    request.request("/api/search/suggest/multimatch", data, create_option(&query, "weapi")).await
}
