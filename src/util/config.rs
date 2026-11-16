//! 对应 api-enhanced 的 `util/config.json`（`APP_CONF` 与 `resourceTypeMap`）。

pub const API_DOMAIN: &str = "https://interface.music.163.com";
pub const EAPI_DOMAIN: &str = "https://interfacepc.music.163.com";
pub const XEAPI_DOMAIN: &str = "https://interface3.music.163.com";
pub const DOMAIN: &str = "https://music.163.com";

/// 对应 `APP_CONF.encrypt`：为 true 时默认加密方式是 eapi，否则是明文 api。
pub const ENCRYPT: bool = true;
/// 对应 `APP_CONF.encryptResponse`：是否让服务器返回加密的响应体（`e_r`）。
pub const ENCRYPT_RESPONSE: bool = false;

/// 对应 `resourceTypeMap`（评论资源类型映射）
pub fn resource_type(t: &str) -> Option<&'static str> {
    Some(match t {
        "0" => "R_SO_4_",
        "1" => "R_MV_5_",
        "2" => "A_PL_0_",
        "3" => "R_AL_3_",
        "4" => "A_DJ_1_",
        "5" => "R_VI_62_",
        "6" => "A_EV_2_",
        "7" => "A_DR_14_",
        _ => return None,
    })
}
