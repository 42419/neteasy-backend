//! 移植自 api-enhanced 的 `module/song_url_v1.js`。
use crate::util::index::{cookie_to_json, obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

/// 本移植版不支持 `unblock=true`（unblockmusic-utils 解灰）。
pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let mut data = obj(json!({
        "ids": format!("[{}]", query.js("id")),
        "level": query.js("level"),
        "encodeType": "flac",
    }));
    let mut options = create_option(&query, "xeapi");
    let level = query.get("level").unwrap_or("");
    if level == "sky" {
        data.insert("immerseType".into(), json!(query.truthy("immerseType").unwrap_or("c51")));
    }
    if level == "vivid" {
        data.insert("encodeType".into(), json!("mp3"));
        options.cookie.insert("os".into(), "android".into());
        options.cookie.insert("appver".into(), "9.5.61".into());
        let _ = cookie_to_json; // 这里 cookie 已经是对象（上游会把字符串 cookie 先转成对象）
    }
    request.request("/api/song/enhance/player/url/v1", data, options).await
}
