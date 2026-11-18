//! 移植自 api-enhanced 的 `module/top_playlist.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{Answer, AnswerResult, Request};
use serde_json::{json, Value};

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    // cat 可选：全部、华语、欧美、日语、韩语、粤语、流行、摇滚、民谣、电子、说唱、轻音乐、爵士……
    let data = obj(json!({
        "cat": query.or("cat", json!("全部")),
        "order": query.or("order", json!("hot")), // hot / new
        "limit": query.or("limit", json!(50)),
        "offset": query.or("offset", json!(0)),
        "total": true,
    }));
    let res = request.request("/api/playlist/list", data, create_option(&query, "weapi")).await?;
    // 上游写法：JSON.stringify(res).replace(/avatarImgId_str/g, 'avatarImgIdStr')
    let text = res.body.to_string().replace("avatarImgId_str", "avatarImgIdStr");
    let body: Value = serde_json::from_str(&text).unwrap_or(res.body);
    Ok(Answer::new(res.status, body, res.cookie))
}
