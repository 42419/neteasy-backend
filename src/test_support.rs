//! 测试支持（只在测试时编译）：本地假的网易云服务，记录收到的请求。

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

/// 假服务收到的一个请求
#[derive(Clone, Debug, Default)]
pub struct Captured {
    /// 请求路径，比如 `/weapi/v3/song/detail`
    pub path: String,
    /// 请求头里的 X-Real-IP（没有则为空）
    pub real_ip: String,
    /// 表单 body 解码后的键值对（weapi 是 params / encSecKey，eapi 是 params，xeapi 是 C / S / R）
    pub form: Vec<(String, String)>,
}

impl Captured {
    pub fn field(&self, name: &str) -> Option<&str> {
        self.form
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

pub type Seen = Arc<Mutex<Vec<Captured>>>;

/// 启动本地假服务：按顺序返回预设的 JSON（用完后一直重复最后一条），并记录每个请求。
/// 返回 (监听端口, 记录列表)。把端口拼成 `http://127.0.0.1:端口` 传给请求的 `domain` 选项即可。
pub fn fake_netease(responses: Vec<&'static str>) -> (u16, Seen) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let seen2 = seen.clone();
    std::thread::spawn(move || {
        let mut n = 0usize;
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            // 读到请求头结束，再把 body 读完
            let mut buf = Vec::new();
            let mut tmp = [0u8; 4096];
            let header_end = loop {
                let k = stream.read(&mut tmp).unwrap_or(0);
                if k == 0 {
                    break None;
                }
                buf.extend_from_slice(&tmp[..k]);
                if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    break Some(i + 4);
                }
            };
            let Some(end) = header_end else { continue };
            let head = String::from_utf8_lossy(&buf[..end]).to_string();
            let len = head
                .lines()
                .find_map(|l| {
                    l.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                })
                .unwrap_or(0);
            while buf.len() < end + len {
                let k = stream.read(&mut tmp).unwrap_or(0);
                if k == 0 {
                    break;
                }
                buf.extend_from_slice(&tmp[..k]);
            }
            let path = head
                .lines()
                .next()
                .and_then(|l| l.split_whitespace().nth(1))
                .unwrap_or_default()
                .to_string();
            let real_ip = head
                .lines()
                .find_map(|l| {
                    l.to_ascii_lowercase()
                        .strip_prefix("x-real-ip:")
                        .map(|v| v.trim().to_string())
                })
                .unwrap_or_default();
            let body = String::from_utf8_lossy(&buf[end..]).to_string();
            let form =
                serde_urlencoded::from_str::<Vec<(String, String)>>(&body).unwrap_or_default();
            seen2.lock().unwrap().push(Captured {
                path,
                real_ip,
                form,
            });

            let reply = responses[n.min(responses.len() - 1)];
            n += 1;
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                reply.len(),
                reply
            );
        }
    });
    (port, seen)
}
