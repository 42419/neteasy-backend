//! Port of api-enhanced `module/comment_new.js`.
use crate::util::config::resource_type;
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{AnswerResult, Request};
use serde_json::{json, Value};

fn num(v: &Value) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.trim().parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let rtype = query.get("type").and_then(resource_type).map(|s| s.to_string());
    let thread_id = format!("{}{}", rtype.clone().unwrap_or_else(|| "undefined".into()), query.js("id"));
    let page_size = query.or("pageSize", json!(20));
    let page_no = query.or("pageNo", json!(1));
    let mut sort_type = query.get("sortType").and_then(|s| s.parse::<f64>().ok()).filter(|n| *n != 0.0).unwrap_or(99.0) as i64;
    if sort_type == 1 {
        sort_type = 99;
    }
    let offset = ((num(&page_no) - 1.0) * num(&page_size)) as i64;
    // 99: recommended, 2: hot, 3: time
    let cursor: Value = match sort_type {
        99 => json!(offset),
        2 => json!(format!("normalHot#{offset}")),
        3 => json!(query.truthy("cursor").unwrap_or("0")),
        _ => json!(""),
    };
    let data = obj(json!({
        "threadId": thread_id,
        "pageNo": page_no,
        "showInner": query.or("showInner", json!(true)),
        "pageSize": page_size,
        "cursor": cursor,
        "sortType": sort_type,
    }));
    request.request("/api/v2/resource/comments", data, create_option(&query, "")).await
}
