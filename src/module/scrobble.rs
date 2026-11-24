//! 移植自 api-enhanced 的 `module/scrobble.js`。
//! 听歌打点：播放时上报，网易云才会把这首歌记进「最近播放」「听歌排行」和累计听歌数据。
//!
//! 和上游一样分两步：先发 `startplay`（进「最近播放」），再发 `play`（带播放秒数，涨听歌计数）。
//! 参数：`id` 歌曲 id，`sourceid` 来源（歌单 id、专辑 id 等），`time` 实际播放的秒数。
//! 使用 `os=osx` 的 cookie 和 clientlog 域名，这是上游验证过能被服务器接受的组合。
use crate::util::config::CL_DOMAIN;
use crate::util::index::{cookie_to_json, obj, Query};
use crate::util::option::create_option;
use crate::util::request::{Answer, AnswerResult, Request};
use serde_json::{json, Value};

pub async fn handle(mut query: Query, request: &Request) -> AnswerResult {
    // 上游：cookie = Object.assign({ os: 'osx' }, cookie)，没有 cookie 时回退到环境变量 NETEASE_COOKIE
    if query.cookie.is_empty() {
        if let Ok(c) = std::env::var("NETEASE_COOKIE") {
            query.cookie = cookie_to_json(&c);
        }
    }
    query.cookie.entry("os".to_string()).or_insert_with(|| "osx".to_string());

    let content = format!("id={}", query.js("sourceid"));
    // 1) startplay → 进「最近播放」
    let startplay = json!([{
        "action": "startplay",
        "json": Value::Object(obj(json!({
            "id": query.val("id"),
            "type": "song",
            "mainsite": "1",
            "mainsiteWeb": "1",
            "content": content,
        }))),
    }]);
    // 2) play → 涨「听歌排行」计数
    let play = json!([{
        "action": "play",
        "json": Value::Object(obj(json!({
            "download": 0,
            "end": "playend",
            "id": query.val("id"),
            "sourceId": query.val("sourceid"),
            "time": query.val("time"),
            "type": "song",
            "wifi": 0,
            "source": "list",
            "mainsite": "1",
            "mainsiteWeb": "1",
            "content": content,
        }))),
    }]);

    let mut option = create_option(&query, "eapi");
    if option.domain.is_empty() {
        option.domain = CL_DOMAIN.to_string();
    }
    // 两次请求按顺序发送；任何一次失败都按错误返回
    let res1 = request
        .request("/api/feedback/weblog", obj(json!({ "logs": startplay.to_string() })), option.clone())
        .await?;
    let res2 = request
        .request("/api/feedback/weblog", obj(json!({ "logs": play.to_string() })), option)
        .await?;

    Ok(Answer::new(
        200,
        json!({
            "code": 200,
            "data": "success",
            "details": { "startplay": res1.body, "play": res2.body },
        }),
        vec![],
    ))
}
