//! 网易云请求/响应的加解密：weapi、eapi、xeapi（移植自 api-enhanced 的 `util/crypto.js`）。
//!
//! 所有随机量都可以注入（`*_with` 系列函数），这样输出能和 Node 实现逐字节对拍
//! （见 `scripts/gen_vectors.js` 和 `tests/crypto_vectors.rs`）。

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyInit, KeyIvInit};
use aes_gcm::{aead::Aead, Aes128Gcm, Nonce};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use num_bigint::BigUint;
use rand::RngCore;
use sha2::Sha256;
use std::io::Read;
use x25519_dalek::{PublicKey, StaticSecret};

type HmacSha256 = Hmac<Sha256>;

pub const IV: &[u8; 16] = b"0102030405060708";
pub const PRESET_KEY: &[u8; 16] = b"0CoJUm6Qyw8W8jud";
pub const EAPI_KEY: &[u8; 16] = b"e82ckenh8dichen8";
const BASE62: &[u8; 62] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const RSA_N_HEX: &str = "e0b509f6259df8642dbc35662901477df22677ec152b5ff68ace615bb7b725152b3ab17a876aea8a5aa76d2e417629ec4ee341f56135fccf695280104e0312ecbda92557c93870114af6c9d05c4f7f0c3685b7a46bee255932575cce10b424d813cfe4875d3e82047b97ddef52741d546b8e289dc6935b3ece0462db0a22b8e7";
const RSA_E: u32 = 65537;
const XEAPI_STATIC_KEY_HEX: &str =
    "ab1d5a430f6bb04a3f01e81ddd72bd916d5ce591248ac128714806d7f8fb1b84";
const XEAPI_SIGN_KEY: &str =
    "mUHCwVNWJbunMqAHf5MImuirT6plvs6VSFW62MGHstFQxhBGdEoIhLItH3djc4+FB/OKty3+lL2rGeoFBpVe5g==";

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("invalid aes key length {0}")]
    KeyLen(usize),
    #[error("decrypt failed")]
    Decrypt,
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("base64: {0}")]
    B64(#[from] base64::DecodeError),
}

pub type Result<T> = std::result::Result<T, CryptoError>;

fn xeapi_static_key() -> Vec<u8> {
    hex::decode(XEAPI_STATIC_KEY_HEX).expect("static key hex")
}

// ---------------------------------------------------------------- AES 辅助函数

pub fn aes_cbc_encrypt_b64(plain: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> String {
    let ct = cbc::Encryptor::<aes::Aes128>::new(key.into(), iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plain);
    STANDARD.encode(ct)
}

pub fn aes_ecb_encrypt(key: &[u8], plain: &[u8]) -> Result<Vec<u8>> {
    Ok(match key.len() {
        16 => ecb::Encryptor::<aes::Aes128>::new(key.into()).encrypt_padded_vec_mut::<Pkcs7>(plain),
        24 => ecb::Encryptor::<aes::Aes192>::new(key.into()).encrypt_padded_vec_mut::<Pkcs7>(plain),
        32 => ecb::Encryptor::<aes::Aes256>::new(key.into()).encrypt_padded_vec_mut::<Pkcs7>(plain),
        n => return Err(CryptoError::KeyLen(n)),
    })
}

pub fn aes_ecb_decrypt(key: &[u8], cipher: &[u8]) -> Result<Vec<u8>> {
    let r =
        match key.len() {
            16 => ecb::Decryptor::<aes::Aes128>::new(key.into())
                .decrypt_padded_vec_mut::<Pkcs7>(cipher),
            24 => ecb::Decryptor::<aes::Aes192>::new(key.into())
                .decrypt_padded_vec_mut::<Pkcs7>(cipher),
            32 => ecb::Decryptor::<aes::Aes256>::new(key.into())
                .decrypt_padded_vec_mut::<Pkcs7>(cipher),
            n => return Err(CryptoError::KeyLen(n)),
        };
    r.map_err(|_| CryptoError::Decrypt)
}

fn md5_hex(s: &str) -> String {
    hex::encode(Md5::digest(s.as_bytes()))
}

// ---------------------------------------------------------------- weapi

pub struct WeapiOut {
    pub params: String,
    pub enc_sec_key: String,
}

/// 用网易云公钥做无填充的原始 RSA，输出左侧补零的十六进制（与 node-forge 的 `NONE` 一致）。
fn rsa_encrypt_raw_hex(msg: &[u8]) -> String {
    let n = BigUint::parse_bytes(RSA_N_HEX.as_bytes(), 16).expect("rsa n");
    let m = BigUint::from_bytes_be(msg);
    let c = m.modpow(&BigUint::from(RSA_E), &n);
    let mut h = c.to_str_radix(16);
    let width = RSA_N_HEX.len();
    while h.len() < width {
        h.insert(0, '0');
    }
    h
}

#[cfg(test)]
thread_local! {
    /// 测试专用：固定 weapi 的随机密钥，这样测试可以解密 `params` 检查请求内容
    pub static TEST_SECRET: std::cell::Cell<Option<[u8; 16]>> = const { std::cell::Cell::new(None) };
}

pub fn random_secret_key() -> [u8; 16] {
    #[cfg(test)]
    if let Some(k) = TEST_SECRET.with(|c| c.get()) {
        return k;
    }
    let mut rng = rand::thread_rng();
    let mut k = [0u8; 16];
    for b in k.iter_mut() {
        *b = BASE62[(rng.next_u32() % 62) as usize];
    }
    k
}

pub fn weapi_with(json_text: &str, secret: &[u8; 16]) -> WeapiOut {
    let inner = aes_cbc_encrypt_b64(json_text.as_bytes(), PRESET_KEY, IV);
    let params = aes_cbc_encrypt_b64(inner.as_bytes(), secret, IV);
    let mut rev = *secret;
    rev.reverse();
    WeapiOut {
        params,
        enc_sec_key: rsa_encrypt_raw_hex(&rev),
    }
}

pub fn weapi(json_text: &str) -> WeapiOut {
    weapi_with(json_text, &random_secret_key())
}

// ---------------------------------------------------------------- eapi

/// 返回大写十六进制的 `params`。
pub fn eapi(url: &str, json_text: &str) -> String {
    let digest = md5_hex(&format!("nobody{url}use{json_text}md5forencrypt"));
    let data = format!("{url}-36cd479b6b5-{json_text}-36cd479b6b5-{digest}");
    hex::encode_upper(aes_ecb_encrypt(EAPI_KEY, data.as_bytes()).expect("eapi key"))
}

fn gunzip(b: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(b).read_to_end(&mut out)?;
    Ok(out)
}

/// 解密 eapi 响应体（原始字节）。`aeapi` 表示服务器先对明文做了 gzip 压缩。
pub fn eapi_res_decrypt(body: &[u8], aeapi: bool) -> Result<serde_json::Value> {
    let dec = aes_ecb_decrypt(EAPI_KEY, body)?;
    let plain = if aeapi { gunzip(&dec)? } else { dec };
    Ok(serde_json::from_slice(&plain)?)
}

// ---------------------------------------------------------------- xeapi

#[derive(Clone, Debug, Default)]
pub struct XeapiPublicKey {
    pub public_key_b64: String, // `publicKey` 字段（原始 32 字节 X25519 公钥，base64）
    pub version: String,
    pub sk: String,
}

#[derive(Clone, Debug, Default)]
pub struct XeapiOptions {
    pub method: Option<String>,       // 默认 POST
    pub content_type: Option<String>, // 默认 form-urlencoded
    pub os: String,                   // 默认 "android"
    pub session_key: Option<Vec<u8>>, // 上一次响应里的 x-encr-sskey
    pub session_id: String,           // x-encr-ssid
}

pub struct XeapiRand {
    pub dynamic_key: [u8; 16],
    pub mid_random: [u8; 16],
    pub eph_secret: [u8; 32],
    pub gcm_iv: [u8; 12],
}

impl XeapiRand {
    pub fn fresh() -> Self {
        let mut r = rand::thread_rng();
        let mut x = XeapiRand {
            dynamic_key: [0; 16],
            mid_random: [0; 16],
            eph_secret: [0; 32],
            gcm_iv: [0; 12],
        };
        r.fill_bytes(&mut x.dynamic_key);
        r.fill_bytes(&mut x.mid_random);
        r.fill_bytes(&mut x.eph_secret);
        r.fill_bytes(&mut x.gcm_iv);
        x
    }
}

pub struct XeapiOut {
    pub c: String, // base64url（不带填充）
    pub s: String, // base64
    pub r: String, // base64
}

fn hmac(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut m = <HmacSha256 as Mac>::new_from_slice(key).expect("hmac key");
    m.update(data);
    m.finalize().into_bytes().to_vec()
}

pub fn xeapi_sign(timestamp: &str, nonce: &str) -> String {
    STANDARD.encode(hmac(
        XEAPI_SIGN_KEY.as_bytes(),
        format!("{timestamp}{nonce}").as_bytes(),
    ))
}

fn xeapi_mid_transform(ct: &[u8], random: &[u8; 16]) -> Vec<u8> {
    let xored: Vec<u8> = ct
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ random[i & 0x0f])
        .collect();
    let rot = if xored.is_empty() {
        0
    } else {
        (random[0] as usize & 0x0f) % xored.len()
    };
    let mut out = Vec::with_capacity(16 + xored.len());
    out.extend_from_slice(random);
    out.extend_from_slice(&xored[rot..]);
    out.extend_from_slice(&xored[..rot]);
    out
}

fn derive_x25519_aes_key(shared: &[u8], eph_pub: &[u8]) -> [u8; 16] {
    let zero = [0u8; 32];
    let prk = hmac(&zero, if shared.is_empty() { &zero } else { shared });
    let mut msg = eph_pub.to_vec();
    msg.push(1);
    let okm = hmac(&prk, &msg);
    let mut k = [0u8; 16];
    k.copy_from_slice(&okm[..16]);
    k
}

fn xeapi_encrypt_s(
    dynamic_key: &[u8],
    pk: &XeapiPublicKey,
    os: &str,
    rnd: &XeapiRand,
) -> Result<Vec<u8>> {
    let peer_raw = STANDARD.decode(&pk.public_key_b64)?;
    let mut peer = [0u8; 32];
    if peer_raw.len() != 32 {
        return Err(CryptoError::KeyLen(peer_raw.len()));
    }
    peer.copy_from_slice(&peer_raw);
    let secret = StaticSecret::from(rnd.eph_secret);
    let eph_pub = PublicKey::from(&secret);
    let shared = secret.diffie_hellman(&PublicKey::from(peer));
    let aes_key = derive_x25519_aes_key(shared.as_bytes(), eph_pub.as_bytes());
    let plaintext = format!("{}|{}|{}", STANDARD.encode(dynamic_key), os, pk.sk);
    let cipher = Aes128Gcm::new(&aes_key.into());
    // aes-gcm 返回「密文 || tag」，与 Node 产出的布局（cipher + authTag）一致
    let enc = cipher
        .encrypt(Nonce::from_slice(&rnd.gcm_iv), plaintext.as_bytes())
        .map_err(|_| CryptoError::Decrypt)?;
    let mut out = Vec::with_capacity(32 + 12 + enc.len());
    out.extend_from_slice(eph_pub.as_bytes());
    out.extend_from_slice(&rnd.gcm_iv);
    out.extend_from_slice(&enc);
    Ok(out)
}

fn json_str(s: &str) -> String {
    serde_json::to_string(s).expect("string json")
}

/// 构造 xeapi 内层明文 JSON。字段顺序与 JS 对象的插入顺序一致（见下方注释），
/// 这样密文才能和 Node 的结果逐字节相同。
pub fn build_xeapi_plaintext(uri: &str, body: &[(String, String)], o: &XeapiOptions) -> String {
    let content_type = o
        .content_type
        .clone()
        .unwrap_or_else(|| "application/x-www-form-urlencoded;charset=utf-8".into());
    let media = content_type.split(';').next().unwrap_or("").to_lowercase();
    let mut parts: Vec<String> = Vec::new();
    if media != "application/x-www-form-urlencoded" {
        parts.push(format!("\"contentType\":{}", json_str(&content_type)));
    }
    let method = o
        .method
        .clone()
        .unwrap_or_else(|| "POST".into())
        .to_uppercase();
    if method != "POST" {
        parts.push(format!("\"method\":{}", json_str(&method)));
    }
    // JS 的插入顺序：只有 uri 本身带 query 时 queryString 才会先赋值；
    // 否则它是在 `content` 之后才赋值（`fields.queryString = 'e_r=true'`）。
    let query = uri
        .split_once('?')
        .map(|(_, q)| q.to_string())
        .unwrap_or_default();
    let has_query = !query.is_empty();
    let query = if has_query {
        format!("{query}&e_r=true")
    } else {
        "e_r=true".to_string()
    };
    if has_query {
        parts.push(format!("\"queryString\":{}", json_str(&query)));
    }
    let pairs: Vec<(&str, &str)> = body
        .iter()
        .filter(|(k, _)| k != "e_r")
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let encoded = serde_urlencoded::to_string(&pairs).unwrap_or_default();
    if !encoded.is_empty() {
        parts.push(format!("\"content\":{}", json_str(&encoded)));
    }
    if !has_query {
        parts.push(format!("\"queryString\":{}", json_str(&query)));
    }
    format!("{{{}}}", parts.join(","))
}

pub fn xeapi_with(
    uri: &str,
    body: &[(String, String)],
    o: &XeapiOptions,
    pk: &XeapiPublicKey,
    rnd: &XeapiRand,
) -> Result<XeapiOut> {
    let session_key = o.session_key.as_ref().filter(|k| !k.is_empty());
    let dynamic_key: Vec<u8> = session_key
        .cloned()
        .unwrap_or_else(|| rnd.dynamic_key.to_vec());
    let plaintext = build_xeapi_plaintext(uri, body, o);
    let inner = aes_ecb_encrypt(&xeapi_static_key(), plaintext.as_bytes())?;
    let c = aes_ecb_encrypt(&dynamic_key, &xeapi_mid_transform(&inner, &rnd.mid_random))?;
    let os = if o.os.is_empty() { "android" } else { &o.os };
    let s = xeapi_encrypt_s(&dynamic_key, pk, os, rnd)?;
    let sid = if session_key.is_some() {
        o.session_id.as_str()
    } else {
        ""
    };
    let r = aes_ecb_encrypt(
        &xeapi_static_key(),
        format!("{}|{}", pk.version, sid).as_bytes(),
    )?;
    Ok(XeapiOut {
        c: URL_SAFE_NO_PAD.encode(c),
        s: STANDARD.encode(s),
        r: STANDARD.encode(r),
    })
}

pub fn xeapi(
    uri: &str,
    body: &[(String, String)],
    o: &XeapiOptions,
    pk: &XeapiPublicKey,
) -> Result<XeapiOut> {
    xeapi_with(uri, body, o, pk, &XeapiRand::fresh())
}

pub fn xeapi_res_decrypt(body: &[u8]) -> Result<serde_json::Value> {
    let dec = aes_ecb_decrypt(EAPI_KEY, body)?;
    let plain = if dec.len() > 1 && dec[0] == 0x1f && dec[1] == 0x8b {
        gunzip(&dec)?
    } else {
        dec
    };
    Ok(serde_json::from_slice(&plain)?)
}

/// 把 `/api/bsr/sk/get` 返回的 `encryptedData` 解密成公钥状态 `{publicKey, version, sk, ...}`。
pub fn xeapi_decrypt_public_key(encrypted_b64: &str) -> Result<serde_json::Value> {
    let raw = STANDARD.decode(encrypted_b64)?;
    let dec = aes_ecb_decrypt(&xeapi_static_key(), &raw)?;
    Ok(serde_json::from_slice(&dec)?)
}

/// 测试专用的解密辅助：把请求体还原成明文 JSON，用来逐个核对各接口发出的参数。
#[cfg(test)]
pub mod test_decrypt {
    use super::*;

    fn cbc_decrypt(key: &[u8; 16], data: &[u8]) -> Vec<u8> {
        cbc::Decryptor::<aes::Aes128>::new(key.into(), IV.into())
            .decrypt_padded_vec_mut::<Pkcs7>(data)
            .expect("cbc decrypt")
    }

    /// weapi：需要事先用 [`TEST_SECRET`] 固定过密钥
    pub fn weapi_params(params_b64: &str, secret: &[u8; 16]) -> serde_json::Value {
        let inner_b64 = cbc_decrypt(secret, &STANDARD.decode(params_b64).expect("b64"));
        let plain = cbc_decrypt(PRESET_KEY, &STANDARD.decode(inner_b64).expect("inner b64"));
        serde_json::from_slice(&plain).expect("weapi json")
    }

    /// eapi：返回 (url, 明文 JSON)，同时校验 md5 摘要
    pub fn eapi_params(params_hex: &str) -> (String, serde_json::Value) {
        let raw = hex::decode(params_hex).expect("hex");
        let text = String::from_utf8(aes_ecb_decrypt(EAPI_KEY, &raw).expect("ecb")).expect("utf8");
        let parts: Vec<&str> = text.split("-36cd479b6b5-").collect();
        assert_eq!(parts.len(), 3, "eapi 明文格式不对: {text}");
        assert_eq!(
            parts[2],
            md5_hex(&format!("nobody{}use{}md5forencrypt", parts[0], parts[1])),
            "eapi 摘要不匹配"
        );
        (
            parts[0].to_string(),
            serde_json::from_str(parts[1]).expect("eapi json"),
        )
    }
}
