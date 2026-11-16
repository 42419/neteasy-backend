//! Port of api-enhanced `module/song_url_v1.js`.
use crate::util::index::{cookie_to_json, obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::json;

/// `unblock=true` (unblockmusic-utils) is not supported in this port.
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
        let _ = cookie_to_json; // cookie already an object here (upstream normalises string cookies)
    }
    request.request("/api/song/enhance/player/url/v1", data, options).await
}
