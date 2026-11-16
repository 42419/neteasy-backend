//! Port of api-enhanced `module/comment_like.js`.
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
        "commentId": query.val("cid"),
    }));
    if rtype == Some("A_EV_2_") {
        data.insert("threadId".into(), json!(query.val("threadId")));
    }
    request.request(&format!("/api/v1/comment/{t}"), data, create_option(&query, "weapi")).await
}
