//! 移植自 api-enhanced 的 `module/homepage_block_page.js`。
//! 官方首页的区块列表：推荐歌单、云村热播、雷达歌单、有声书……都由它按官方顺序返回。
//! 翻页时把上一页返回的 `cursor` 带回来。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({
        "refresh": query.or("refresh", json!(false)),
        "cursor": query.val("cursor"),
    }));
    request.request("/api/homepage/block/page", data, create_option(&query, "weapi")).await
}
