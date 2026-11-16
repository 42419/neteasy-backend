//! Port of api-enhanced `module/login_qr_check.js`.
use crate::util::index::{obj, Query};
use crate::util::option::create_option;
use crate::util::request::{Answer, AnswerResult, Request};
use serde_json::{json, Value};

pub async fn handle(query: Query, request: &Request) -> AnswerResult {
    let data = obj(json!({ "key": query.val("key"), "type": 3 }));
    match request.request("/api/login/qrcode/client/login", data, create_option(&query, "")).await {
        Ok(result) => {
            let mut body = result.body.clone();
            if let Value::Object(m) = &mut body {
                m.insert("cookie".into(), Value::String(result.cookie.join(";")));
            }
            Ok(Answer::new(200, body, result.cookie))
        }
        // upstream's catch block dereferences an out-of-scope `result` (ReferenceError), so errors
        // surface through the server as 404 "Not Found"; keep that observable behaviour.
        Err(_) => Err(Answer::new(500, Value::Null, vec![])),
    }
}
