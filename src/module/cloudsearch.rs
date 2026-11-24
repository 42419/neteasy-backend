//! 移植自 api-enhanced 的 `module/cloudsearch.js`。
//! 搜索（官方客户端搜索页用的版本，比 `search` 返回得更全，带歌曲权限信息）。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "s": query.val("keywords"),
        // 1 单曲、10 专辑、100 歌手、1000 歌单、1002 用户、1004 MV、1006 歌词、1009 播客、1014 视频
        "type": query.or("type", json!(1)),
        "limit": query.or("limit", json!(30)),
        "offset": query.or("offset", json!(0)),
        "total": true,
    }));
    request.request("/api/cloudsearch/pc", data, create_option(&query, "")).await
}
