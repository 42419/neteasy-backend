//! Mirrors api-enhanced `util/config.json` (`APP_CONF` + `resourceTypeMap`).

pub const API_DOMAIN: &str = "https://interface.music.163.com";
pub const EAPI_DOMAIN: &str = "https://interfacepc.music.163.com";
pub const XEAPI_DOMAIN: &str = "https://interface3.music.163.com";
pub const DOMAIN: &str = "https://music.163.com";

/// `APP_CONF.encrypt`: default crypto is eapi when true, plain api when false.
pub const ENCRYPT: bool = true;
/// `APP_CONF.encryptResponse`: ask the server for an encrypted response body (`e_r`).
pub const ENCRYPT_RESPONSE: bool = false;

/// `resourceTypeMap`
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
