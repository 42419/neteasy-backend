//! 移植自 api-enhanced 的 `module/user_detail.js`。
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{Answer, AnswerResult, Request};
use serde_json::{json, Value};

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let res = request
        .request(&format!("/api/v1/user/detail/{}", query.js("uid")), obj(json!({})), create_option(&query, "weapi"))
        .await?;
    // 上游写法：JSON.stringify(res).replace(/avatarImgId_str/g, 'avatarImgIdStr')
    let text = res.body.to_string().replace("avatarImgId_str", "avatarImgIdStr");
    let body: Value = serde_json::from_str(&text).unwrap_or(res.body);
    Ok(Answer::new(res.status, body, res.cookie))
}
