//! Port of api-enhanced `module/login_qr_create.js`.
use crate::util::index::{generate_chain_id, Query};
use crate::util::request::{Answer, AnswerResult, Request};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::json;

/// `QRCode.toDataURL(url)`: PNG data URL.
fn qr_data_url(text: &str) -> String {
    let Ok(code) = qrcode::QrCode::new(text.as_bytes()) else { return String::new() };
    let w = code.width();
    let colors = code.to_colors();
    let (scale, margin) = (8usize, 4usize);
    let size = (w + 2 * margin) * scale;
    let mut pix = vec![255u8; size * size];
    for y in 0..w {
        for x in 0..w {
            if colors[y * w + x] == qrcode::Color::Dark {
                for dy in 0..scale {
                    for dx in 0..scale {
                        pix[((y + margin) * scale + dy) * size + (x + margin) * scale + dx] = 0;
                    }
                }
            }
        }
    }
    let mut buf = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut buf, size as u32, size as u32);
        enc.set_color(png::ColorType::Grayscale);
        enc.set_depth(png::BitDepth::Eight);
        let Ok(mut writer) = enc.write_header() else { return String::new() };
        if writer.write_image_data(&pix).is_err() {
            return String::new();
        }
    }
    format!("data:image/png;base64,{}", STANDARD.encode(buf))
}

pub async fn handle(query: Query, _request: &Request) -> AnswerResult {
    let platform = query.truthy("platform").unwrap_or("pc");
    let mut url = format!("https://music.163.com/login?codekey={}", query.js("key"));
    if platform == "web" {
        url.push_str(&format!("&chainId={}", generate_chain_id(&query.cookie)));
    }
    let qrimg = if query.truthy("qrimg").is_some() { qr_data_url(&url) } else { String::new() };
    Ok(Answer::new(200, json!({ "code": 200, "data": { "qrurl": url, "qrimg": qrimg } }), vec![]))
}
