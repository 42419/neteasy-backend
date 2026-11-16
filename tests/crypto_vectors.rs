//! Byte-for-byte cross-checks against api-enhanced `util/crypto.js` (vectors from scripts/gen_vectors.js).
use neteasy_backend::util::crypto::*;
use serde_json::Value;

fn vectors() -> Value {
    serde_json::from_str(include_str!("vectors.json")).expect("vectors.json")
}

fn h16(s: &str) -> [u8; 16] {
    hex::decode(s).unwrap().try_into().unwrap()
}
fn h12(s: &str) -> [u8; 12] {
    hex::decode(s).unwrap().try_into().unwrap()
}
fn h32(s: &str) -> [u8; 32] {
    hex::decode(s).unwrap().try_into().unwrap()
}

#[test]
fn weapi_matches_node() {
    let v = &vectors()["weapi"];
    let secret: [u8; 16] = v["secret"].as_str().unwrap().as_bytes().try_into().unwrap();
    let out = weapi_with(v["text"].as_str().unwrap(), &secret);
    assert_eq!(out.params, v["params"].as_str().unwrap());
    assert_eq!(out.enc_sec_key, v["encSecKey"].as_str().unwrap());
}

#[test]
fn eapi_matches_node() {
    let v = &vectors()["eapi"];
    assert_eq!(
        eapi(v["url"].as_str().unwrap(), v["text"].as_str().unwrap()),
        v["params"].as_str().unwrap()
    );
}

#[test]
fn eapi_response_decrypt() {
    let v = &vectors()["eapi_res"];
    let want: Value = serde_json::from_str(v["plain"].as_str().unwrap()).unwrap();
    assert_eq!(
        eapi_res_decrypt(&hex::decode(v["enc_hex"].as_str().unwrap()).unwrap(), false).unwrap(),
        want
    );
    assert_eq!(
        eapi_res_decrypt(
            &hex::decode(v["gz_enc_hex"].as_str().unwrap()).unwrap(),
            true
        )
        .unwrap(),
        want
    );
}

#[test]
fn xeapi_response_decrypt_autodetects_gzip() {
    let v = &vectors()["xeapi_res"];
    let want: Value = serde_json::from_str(v["plain"].as_str().unwrap()).unwrap();
    assert_eq!(
        xeapi_res_decrypt(&hex::decode(v["enc_hex"].as_str().unwrap()).unwrap()).unwrap(),
        want
    );
    assert_eq!(
        xeapi_res_decrypt(&hex::decode(v["gz_enc_hex"].as_str().unwrap()).unwrap()).unwrap(),
        want
    );
}

#[test]
fn sign_and_public_key_blob() {
    let all = vectors();
    let s = &all["sign"];
    assert_eq!(
        xeapi_sign(s["ts"].as_str().unwrap(), s["nonce"].as_str().unwrap()),
        s["sig"].as_str().unwrap()
    );
    let pk = &all["public_key"];
    assert_eq!(
        xeapi_decrypt_public_key(pk["blob"].as_str().unwrap()).unwrap(),
        pk["decoded"]
    );
}

fn check_xeapi(name: &str) {
    let v = &vectors()[name];
    let pk = XeapiPublicKey {
        public_key_b64: v["public_key"]["publicKey"].as_str().unwrap().into(),
        version: v["public_key"]["version"].as_str().unwrap().into(),
        sk: v["public_key"]["sk"].as_str().unwrap().into(),
    };
    let body: Vec<(String, String)> = v["body"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p[0].as_str().unwrap().to_string(),
                p[1].as_str().unwrap().to_string(),
            )
        })
        .collect();
    let o = &v["opts"];
    let opts = XeapiOptions {
        os: o["os"].as_str().unwrap().into(),
        session_key: o["sessionKey"].as_str().map(|s| s.as_bytes().to_vec()),
        session_id: o["sessionId"].as_str().unwrap().into(),
        ..Default::default()
    };
    let r = &v["rand"];
    let rnd = XeapiRand {
        dynamic_key: h16(r["dynamic_key"].as_str().unwrap()),
        mid_random: h16(r["mid_random"].as_str().unwrap()),
        eph_secret: h32(r["eph_secret"].as_str().unwrap()),
        gcm_iv: h12(r["gcm_iv"].as_str().unwrap()),
    };
    let out = xeapi_with(v["uri"].as_str().unwrap(), &body, &opts, &pk, &rnd).unwrap();
    assert_eq!(out.c, v["C"].as_str().unwrap(), "{name}: C");
    assert_eq!(out.s, v["S"].as_str().unwrap(), "{name}: S");
    assert_eq!(out.r, v["R"].as_str().unwrap(), "{name}: R");
}

#[test]
fn xeapi_basic_matches_node() {
    check_xeapi("xeapi_basic");
}
#[test]
fn xeapi_unicode_and_query_matches_node() {
    check_xeapi("xeapi_unicode");
}
#[test]
fn xeapi_with_session_key_matches_node() {
    check_xeapi("xeapi_session");
}
