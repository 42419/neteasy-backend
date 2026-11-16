//! 对应 api-enhanced 的 `generateConfig.js`：初始化 xeapi 公钥和游客（匿名）token。
//! 在服务开始监听之后于后台运行；引导完成前发来的请求会有限时地等待而不是直接失败，
//! 上一次运行缓存下来的状态可以立即使用。

use crate::module::register_anonimous;
use crate::util::index::{cookie_to_json, generate_device_id, Query};
use crate::util::request::Request;
use crate::util::xeapi_key::get_xeapi_public_key;

pub async fn generate_config(request: &Request) {
    let req = request.for_bootstrap();
    if req.device_id().is_empty() {
        req.set_device_id(&generate_device_id());
    }
    let device_id = req.device_id();

    // 1) xeapi 公钥（其中的 `sk` 与设备 ID 绑定）
    match get_xeapi_public_key(&req, req.xeapi_public_key().as_ref(), &device_id).await {
        Ok(key) => {
            log::info!(
                "[generate_config] xeapi public key ok (version {})",
                key.version
            );
            req.set_xeapi_public_key(&key, &device_id)
        }
        Err(e) => log::warn!("[generate_config] xeapi public key: {e}"),
    }

    // 2) 游客 token（MUSIC_A）
    match register_anonimous::handle(Query::default(), &req).await {
        Ok(answer) => {
            let cookie = answer
                .body
                .get("cookie")
                .and_then(|c| c.as_str())
                .unwrap_or("");
            let parsed = cookie_to_json(&cookie.replace(';', "; "));
            if let Some(token) = parsed.get("MUSIC_A").filter(|t| !t.is_empty()) {
                req.set_anonymous_token(token);
                log::info!("[generate_config] guest token ok");
            } else {
                log::warn!(
                    "[generate_config] anonymous response had no MUSIC_A: {}",
                    answer.body
                );
            }
        }
        Err(e) => log::warn!("[generate_config] anonymous register failed: {}", e.body),
    }
    req.mark_ready();
}
