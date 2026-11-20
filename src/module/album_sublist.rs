//! 移植自 api-enhanced 的 `module/album_sublist.js`。
//! 收藏的专辑。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "limit": query.or("limit", json!(25)),
        "offset": query.or("offset", json!(0)),
        "total": true,
    }));
    request.request("/api/album/sublist", data, create_option(&query, "weapi")).await
}
