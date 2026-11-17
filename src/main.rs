use neteasy_backend::{serve_ncm_api, ServeOptions};

struct StderrLogger;
impl log::Log for StderrLogger {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Info
    }
    fn log(&self, r: &log::Record) {
        if self.enabled(r.metadata()) {
            eprintln!("[{}] {}", r.level(), r.args());
        }
    }
    fn flush(&self) {}
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _ = log::set_logger(&StderrLogger).map(|()| log::set_max_level(log::LevelFilter::Info));
    let mut opts = ServeOptions::default();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--port" => {
                opts.port = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(opts.port)
            }
            "--host" => opts.host = args.next().unwrap_or(opts.host),
            "--data-dir" => opts.data_dir = args.next().map(Into::into).unwrap_or(opts.data_dir),
            // GET 响应缓存有效期（秒），0 关闭
            "--cache-ttl" => {
                opts.cache_ttl_secs = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(opts.cache_ttl_secs)
            }
            "--no-cache" => opts.cache_ttl_secs = 0,
            // 一律用随机中国 IP 填充 X-Real-IP（境外网络使用时开启）
            "--random-cn-ip" => opts.random_cn_ip = true,
            "-h" | "--help" => {
                println!(
                    "neteasy-backend [--host 127.0.0.1] [--port 19800] [--data-dir ./data]\n                  [--cache-ttl 120 | --no-cache] [--random-cn-ip]"
                );
                return Ok(());
            }
            other => eprintln!("ignoring unknown argument: {other}"),
        }
    }
    serve_ncm_api(opts).await
}
