//! 对应 api-enhanced 的 `module/`：一个接口一个文件。文件 `song_url_v1.rs` 对应路由 `/song/url/v1`
//!（路由 = 文件名里的 `_` 换成 `/`，和上游的 `getModulesDefinitions` 一致）。
//!
//! 只移植了 neteasy 应用用到的接口，进度见 README 里的状态表。

use crate::util::index::Query;
use crate::util::request::{AnswerResult, Request};
use std::future::Future;
use std::pin::Pin;

pub type ModuleFuture<'a> = Pin<Box<dyn Future<Output = AnswerResult> + Send + 'a>>;
/// 上游的模块约定：`(query, request) => Promise<{ status, body, cookie }>`。
pub type ModuleFn = for<'a> fn(Query, &'a Request) -> ModuleFuture<'a>;

macro_rules! modules {
    ($($name:ident),* $(,)?) => {
        $(pub mod $name;)*

        /// 每个已移植模块的 `(标识符, 路由, 处理函数)`。
        pub fn registry() -> Vec<(&'static str, String, ModuleFn)> {
            vec![
                $((
                    stringify!($name),
                    format!("/{}", stringify!($name).replace('_', "/")),
                    {
                        fn call<'a>(q: Query, r: &'a Request) -> ModuleFuture<'a> {
                            Box::pin($name::handle(q, r))
                        }
                        call as ModuleFn
                    },
                )),*
            ]
        }
    };
}

modules! {
    album,
    banner,
    comment_like,
    comment_new,
    dj_detail,
    dj_program,
    dj_program_detail,
    dj_program_toplist,
    dj_recommend,
    homepage_block_page,
    homepage_dragon_ball,
    like,
    likelist,
    login_qr_check,
    login_qr_create,
    login_qr_key,
    login_status,
    lyric,
    lyric_new,
    personalized,
    personalized_djprogram,
    personalized_newsong,
    playlist_catlist,
    playlist_detail,
    playlist_hot,
    playlist_track_all,
    program_recommend,
    recommend_songs,
    register_anonimous,
    register_xeapikey,
    search,
    search_hot,
    search_suggest,
    song_detail,
    song_url_v1,
    style_artist,
    style_detail,
    style_list,
    style_playlist,
    style_preference,
    style_song,
    top_artists,
    top_playlist,
    top_playlist_highquality,
    toplist,
    toplist_artist,
    user_detail,
    user_playlist,
    voicelist_detail,
    voicelist_list,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fake_netease;
    use crate::util::crypto::{test_decrypt, TEST_SECRET};
    use serde_json::{json, Value};

    const SECRET: [u8; 16] = *b"abcdefghijklmnop";

    /// 对 `route` 发一次请求到本地假服务，返回 (请求路径, 解密还原后的请求数据)。
    /// 请求里每次都会自动附带的 `e_r` / `csrf_token` / `header` 会被去掉，只留模块自己写的参数。
    async fn call(route: &str, params: &[(&str, &str)]) -> (String, Value) {
        // 固定 weapi 的随机密钥，才能解密 params
        TEST_SECRET.with(|c| c.set(Some(SECRET)));
        let (port, seen) = fake_netease(vec![r#"{"code":200}"#]);
        let dir = std::env::temp_dir().join(format!("neteasy-mod-{}", rand::random::<u32>()));
        let request = Request::new(dir);
        request.mark_ready();
        let mut query = Query::default();
        for (k, v) in params {
            query.params.insert(k.to_string(), v.to_string());
        }
        query
            .params
            .insert("domain".into(), format!("http://127.0.0.1:{port}"));
        let (_, _, handler) = registry()
            .into_iter()
            .find(|(_, r, _)| r == route)
            .expect("路由没有注册");
        handler(query, &request)
            .await
            .unwrap_or_else(|e| panic!("{route} 返回错误: {}", e.body));

        let captured = seen.lock().unwrap()[0].clone();
        let params_field = captured.field("params").expect("缺少 params 字段");
        let mut data = if captured.path.starts_with("/weapi/") {
            test_decrypt::weapi_params(params_field, &SECRET)
        } else if captured.path.starts_with("/eapi/") {
            test_decrypt::eapi_params(params_field).1
        } else {
            panic!("意外的路径: {}", captured.path)
        };
        if let Value::Object(m) = &mut data {
            for k in ["e_r", "csrf_token", "header"] {
                m.remove(k);
            }
        }
        (captured.path, data)
    }

    /// 首页相关的新接口：(路由, 请求参数, 期望的上游路径, 期望的请求数据)
    #[allow(clippy::type_complexity)]
    fn home_cases() -> Vec<(
        &'static str,
        Vec<(&'static str, &'static str)>,
        &'static str,
        Value,
    )> {
        vec![
            // 官方首页区块：没传 cursor 时请求里不能有这个字段（对应 JS 的 undefined）
            (
                "/homepage/block/page",
                vec![],
                "/weapi/homepage/block/page",
                json!({ "refresh": false }),
            ),
            (
                "/homepage/block/page",
                vec![("refresh", "true"), ("cursor", "abc")],
                "/weapi/homepage/block/page",
                json!({ "refresh": "true", "cursor": "abc" }),
            ),
            (
                "/homepage/dragon/ball",
                vec![],
                "/eapi/homepage/dragon/ball/static",
                json!({}),
            ),
            (
                "/program/recommend",
                vec![("type", "3")],
                "/weapi/program/recommend/v1",
                json!({ "cateId": "3", "limit": 10, "offset": 0 }),
            ),
            (
                "/personalized/newsong",
                vec![],
                "/weapi/personalized/newsong",
                json!({ "type": "recommend", "limit": 10, "areaId": 0 }),
            ),
            (
                "/personalized/djprogram",
                vec![],
                "/weapi/personalized/djprogram",
                json!({}),
            ),
            // 标签歌单
            (
                "/top/playlist",
                vec![("cat", "说唱"), ("order", "new"), ("limit", "20")],
                "/weapi/playlist/list",
                json!({ "cat": "说唱", "order": "new", "limit": "20", "offset": 0, "total": true }),
            ),
            (
                "/playlist/catlist",
                vec![],
                "/eapi/playlist/catalogue",
                json!({}),
            ),
            (
                "/playlist/hot",
                vec![],
                "/weapi/playlist/hottags",
                json!({}),
            ),
            (
                "/top/playlist/highquality",
                vec![("before", "123")],
                "/weapi/playlist/highquality/list",
                json!({ "cat": "全部", "limit": 50, "lasttime": "123", "total": true }),
            ),
            // 曲风
            ("/style/list", vec![], "/weapi/tag/list/get", json!({})),
            (
                "/style/detail",
                vec![("tagId", "1000")],
                "/weapi/style-tag/home/head",
                json!({ "tagId": "1000" }),
            ),
            (
                "/style/playlist",
                vec![("tagId", "1000")],
                "/weapi/style-tag/home/playlist",
                json!({ "cursor": 0, "size": 20, "tagId": "1000", "sort": 0 }),
            ),
            (
                "/style/artist",
                vec![("tagId", "1000")],
                "/weapi/style-tag/home/artist",
                json!({ "cursor": 0, "size": 20, "tagId": "1000", "sort": 0 }),
            ),
            (
                "/style/song",
                vec![("tagId", "1000"), ("sort", "1")],
                "/weapi/style-tag/home/song",
                json!({ "cursor": 0, "size": 20, "tagId": "1000", "sort": "1" }),
            ),
            (
                "/style/preference",
                vec![],
                "/weapi/tag/my/preference/get",
                json!({}),
            ),
            // 电台 / 节目
            (
                "/dj/program/detail",
                vec![("id", "9")],
                "/weapi/dj/program/detail",
                json!({ "id": "9" }),
            ),
            (
                "/dj/detail",
                vec![("rid", "7")],
                "/weapi/djradio/v2/get",
                json!({ "id": "7" }),
            ),
            (
                "/dj/program",
                vec![("rid", "7"), ("asc", "true")],
                "/weapi/dj/program/byradio",
                json!({ "radioId": "7", "limit": 30, "offset": 0, "asc": true }),
            ),
            (
                "/dj/program/toplist",
                vec![],
                "/weapi/program/toplist/v1",
                json!({ "limit": 100, "offset": 0 }),
            ),
            (
                "/dj/recommend",
                vec![],
                "/weapi/djradio/recommend/v1",
                json!({}),
            ),
            // 艺人
            (
                "/top/artists",
                vec![],
                "/weapi/artist/top",
                json!({ "limit": 50, "offset": 0, "total": true }),
            ),
            (
                "/toplist/artist",
                vec![("type", "2")],
                "/weapi/toplist/artist",
                json!({ "type": "2", "limit": 100, "offset": 0, "total": true }),
            ),
            // 有声书
            (
                "/voicelist/detail",
                vec![("id", "5")],
                "/eapi/voice/workbench/voicelist/detail",
                json!({ "id": "5" }),
            ),
            (
                "/voicelist/list",
                vec![("voiceListId", "5")],
                "/eapi/voice/workbench/voices/by/voicelist",
                json!({ "limit": "200", "offset": "0", "voiceListId": "5" }),
            ),
        ]
    }

    #[tokio::test]
    async fn home_modules_send_the_expected_path_crypto_and_params() {
        for (route, params, want_path, want_data) in home_cases() {
            let (path, data) = call(route, &params).await;
            assert_eq!(path, want_path, "{route}: 请求路径/加密方式不对");
            assert_eq!(data, want_data, "{route}: 请求参数不对");
        }
    }

    #[test]
    fn every_home_case_is_registered_and_names_follow_the_file_convention() {
        let routes: Vec<String> = registry().into_iter().map(|(_, r, _)| r).collect();
        for (route, ..) in home_cases() {
            assert!(routes.contains(&route.to_string()), "{route} 没有注册");
        }
    }
}
