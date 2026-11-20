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
        $(
            #[allow(non_snake_case)]
            pub mod $name;
        )*

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
    album_sublist,
    artist_sublist,
    banner,
    comment_like,
    comment_new,
    digitalAlbum_purchased,
    dj_detail,
    dj_program,
    dj_program_detail,
    dj_program_toplist,
    dj_recommend,
    dj_sublist,
    homepage_block_page,
    homepage_dragon_ball,
    like,
    likelist,
    listen_data_realtime_report,
    listen_data_report,
    listen_data_song_play_rank,
    listen_data_today_song,
    listen_data_total,
    listen_data_year_report,
    login_qr_check,
    login_qr_create,
    login_qr_key,
    login_status,
    lyric,
    lyric_new,
    mlog_mycollect,
    mv_sublist,
    personalized,
    personalized_djprogram,
    personalized_newsong,
    playlist_catlist,
    playlist_detail,
    playlist_hot,
    playlist_track_all,
    playmode_intelligence_list,
    program_recommend,
    recent_listen_list,
    recommend_songs,
    record_recent_album,
    record_recent_dj,
    record_recent_playlist,
    record_recent_song,
    record_recent_video,
    record_recent_voice,
    register_anonimous,
    register_xeapikey,
    search,
    search_hot,
    search_suggest,
    signin_progress,
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
    user_account,
    user_binding,
    user_cloud,
    user_cloud_detail,
    user_detail,
    user_detail_new,
    user_dj,
    user_followeds,
    user_follows,
    user_level,
    user_medal,
    user_playlist,
    user_playlist_collect,
    user_record,
    user_social_status,
    user_social_status_rcmd,
    user_subcount,
    vip_growthpoint,
    vip_growthpoint_details,
    vip_info,
    vip_info_v2,
    vip_sign_detail,
    vip_sign_history,
    vip_sign_info,
    vip_tasks,
    vip_tasks_v1,
    vip_timemachine,
    voicelist_detail,
    voicelist_list,
    yunbei,
    yunbei_info,
    yunbei_task_list_v1,
    yunbei_tasks,
    yunbei_tasks_todo,
    yunbei_today,
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
        // xeapi 请求需要公钥；这里用一把随便生成的，只为让请求能发出去
        let secret = x25519_dalek::StaticSecret::from([7u8; 32]);
        request.set_xeapi_public_key(
            &crate::util::crypto::XeapiPublicKey {
                public_key_b64: base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    x25519_dalek::PublicKey::from(&secret).as_bytes(),
                ),
                version: "v1".into(),
                sk: "sk".into(),
            },
            "DEV",
        );
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
        // xeapi 的请求体（C / S / R）无法在测试里解密，只能核对路径
        if captured.path.starts_with("/xeapi/") {
            assert!(
                captured.field("C").is_some() && captured.field("S").is_some(),
                "xeapi 缺少 C/S 字段"
            );
            return (captured.path, Value::Null);
        }
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

    /// 「我的」页相关的新接口：(路由, 请求参数, 期望的上游路径, 期望的请求数据；xeapi 无法解密，期望值写 null)
    #[allow(clippy::type_complexity)]
    fn profile_cases() -> Vec<(
        &'static str,
        Vec<(&'static str, &'static str)>,
        &'static str,
        Value,
    )> {
        vec![
            (
                "/user/account",
                vec![],
                "/weapi/nuser/account/get",
                json!({}),
            ),
            (
                "/user/detail/new",
                vec![("uid", "123")],
                "/eapi/w/v1/user/detail/123",
                json!({"all":"true","userId":"123"}),
            ),
            ("/user/level", vec![], "/weapi/user/level", json!({})),
            ("/user/subcount", vec![], "/weapi/subcount", json!({})),
            (
                "/user/follows",
                vec![("uid", "123")],
                "/weapi/user/getfollows/123",
                json!({"offset":0,"limit":30,"order":true}),
            ),
            (
                "/user/followeds",
                vec![("uid", "123")],
                "/eapi/user/getfolloweds/123",
                json!({"userId":"123","time":"0","limit":20,"offset":0,"getcounts":"true"}),
            ),
            (
                "/user/medal",
                vec![("uid", "123")],
                "/eapi/medal/user/page",
                json!({"uid":"123"}),
            ),
            (
                "/user/record",
                vec![("uid", "123")],
                "/weapi/v1/play/record",
                json!({"uid":"123","type":0}),
            ),
            (
                "/user/social/status",
                vec![("uid", "123")],
                "/eapi/social/user/status",
                json!({"visitorId":"123"}),
            ),
            (
                "/user/social/status/rcmd",
                vec![],
                "/eapi/social/user/status/rcmd",
                json!({}),
            ),
            (
                "/user/binding",
                vec![("uid", "123")],
                "/weapi/v1/user/bindings/123",
                json!({}),
            ),
            (
                "/user/playlist/collect",
                vec![("uid", "123")],
                "/eapi/user/playlist/collect",
                json!({"limit":"100","offset":"0","userId":"123","isWebview":"true","includeRedHeart":"true","includeTop":"true"}),
            ),
            (
                "/playmode/intelligence/list",
                vec![("id", "5"), ("pid", "6")],
                "/eapi/playmode/intelligence/list",
                json!({"songId":"5","type":"fromPlayOne","playlistId":"6","startMusicId":"5","count":1}),
            ),
            (
                "/playmode/intelligence/list",
                vec![("id", "5"), ("pid", "6"), ("sid", "9"), ("count", "3")],
                "/eapi/playmode/intelligence/list",
                json!({"songId":"5","type":"fromPlayOne","playlistId":"6","startMusicId":"9","count":"3"}),
            ),
            (
                "/record/recent/song",
                vec![],
                "/weapi/play-record/song/list",
                json!({"limit":100}),
            ),
            (
                "/record/recent/playlist",
                vec![],
                "/weapi/play-record/playlist/list",
                json!({"limit":100}),
            ),
            (
                "/record/recent/album",
                vec![],
                "/weapi/play-record/album/list",
                json!({"limit":100}),
            ),
            (
                "/record/recent/video",
                vec![],
                "/weapi/play-record/newvideo/list",
                json!({"limit":100}),
            ),
            (
                "/record/recent/voice",
                vec![],
                "/weapi/play-record/voice/list",
                json!({"limit":100}),
            ),
            (
                "/record/recent/dj",
                vec![],
                "/weapi/play-record/djradio/list",
                json!({"limit":100}),
            ),
            (
                "/recent/listen/list",
                vec![],
                "/eapi/pc/recent/listen/list",
                json!({}),
            ),
            (
                "/user/cloud",
                vec![],
                "/weapi/v1/cloud/get",
                json!({"limit":30,"offset":0}),
            ),
            (
                "/user/cloud/detail",
                vec![("id", "1, 2 ,3")],
                "/weapi/v1/cloud/get/byids",
                json!({"songIds":["1","2","3"]}),
            ),
            ("/yunbei", vec![], "/weapi/point/signed/get", json!({})),
            ("/yunbei/info", vec![], "/weapi/v1/user/info", json!({})),
            ("/yunbei/today", vec![], "/weapi/point/today/get", json!({})),
            (
                "/yunbei/tasks",
                vec![],
                "/weapi/usertool/task/list/all",
                json!({}),
            ),
            (
                "/yunbei/tasks/todo",
                vec![],
                "/weapi/usertool/task/todo/query",
                json!({}),
            ),
            (
                "/yunbei/task/list/v1",
                vec![],
                "/weapi/ad/power/yunbei/distribution/list",
                json!({}),
            ),
            (
                "/user/dj",
                vec![("uid", "123")],
                "/weapi/dj/program/123",
                json!({"limit":30,"offset":0}),
            ),
            (
                "/dj/sublist",
                vec![],
                "/weapi/djradio/get/subed",
                json!({"limit":30,"offset":0,"total":true}),
            ),
            (
                "/album/sublist",
                vec![],
                "/weapi/album/sublist",
                json!({"limit":25,"offset":0,"total":true}),
            ),
            (
                "/artist/sublist",
                vec![],
                "/weapi/artist/sublist",
                json!({"limit":25,"offset":0,"total":true}),
            ),
            (
                "/mv/sublist",
                vec![],
                "/weapi/cloudvideo/allvideo/sublist",
                json!({"limit":25,"offset":0,"total":true}),
            ),
            (
                "/digitalAlbum/purchased",
                vec![],
                "/weapi/digitalAlbum/purchased",
                json!({"limit":30,"offset":0,"total":true}),
            ),
            (
                "/mlog/mycollect",
                vec![],
                "/xeapi/mlog/mycollect/bytime/get",
                json!(null),
            ),
            (
                "/vip/info",
                vec![("uid", "123")],
                "/weapi/music-vip-membership/front/vip/info",
                json!({"userId":"123"}),
            ),
            (
                "/vip/info",
                vec![],
                "/weapi/music-vip-membership/front/vip/info",
                json!({"userId":""}),
            ),
            (
                "/vip/info/v2",
                vec![("uid", "123")],
                "/weapi/music-vip-membership/client/vip/info",
                json!({"userId":"123"}),
            ),
            (
                "/vip/growthpoint",
                vec![],
                "/weapi/vipnewcenter/app/level/growhpoint/basic",
                json!({}),
            ),
            (
                "/vip/growthpoint/details",
                vec![],
                "/weapi/vipnewcenter/app/level/growth/details",
                json!({"limit":20,"offset":0}),
            ),
            (
                "/vip/tasks",
                vec![],
                "/weapi/vipnewcenter/app/level/task/list",
                json!({}),
            ),
            (
                "/vip/tasks/v1",
                vec![("id", "123")],
                "/xeapi/middle/vip/mission/user/progress/list",
                json!(null),
            ),
            (
                "/vip/timemachine",
                vec![],
                "/weapi/vipmusic/newrecord/weekflow",
                json!({}),
            ),
            (
                "/vip/timemachine",
                vec![("startTime", "1"), ("endTime", "2")],
                "/weapi/vipmusic/newrecord/weekflow",
                json!({"startTime":"1","endTime":"2","type":1,"limit":60}),
            ),
            (
                "/vip/sign/info",
                vec![],
                "/weapi/vipnewcenter/app/user/sign/info",
                json!({}),
            ),
            (
                "/vip/sign/detail",
                vec![("timestamp", "111")],
                "/eapi/vipnewcenter/app/level/user/checkin/history/detail",
                json!({"signDayTime":"111","type":"1"}),
            ),
            (
                "/vip/sign/history",
                vec![],
                "/eapi/vipnewcenter/app/minidesk/music/sign/pc",
                json!({"type":"0"}),
            ),
            (
                "/signin/progress",
                vec![],
                "/weapi/act/modules/signin/v2/progress",
                json!({"moduleId":"1207signin-1207signin"}),
            ),
            (
                "/listen/data/total",
                vec![],
                "/eapi/content/activity/listen/data/total",
                json!({}),
            ),
            (
                "/listen/data/today/song",
                vec![],
                "/eapi/content/activity/listen/data/today/song/play/rank",
                json!({}),
            ),
            (
                "/listen/data/year/report",
                vec![],
                "/eapi/content/activity/listen/data/year/report",
                json!({}),
            ),
            (
                "/listen/data/report",
                vec![],
                "/eapi/content/activity/listen/data/report",
                json!({"type":"week"}),
            ),
            (
                "/listen/data/realtime/report",
                vec![],
                "/eapi/content/activity/listen/data/realtime/report",
                json!({"type":"week"}),
            ),
            (
                "/listen/data/song/play/rank",
                vec![],
                "/eapi/content/activity/listen/data/song/play/rank",
                json!({"type":"month"}),
            ),
        ]
    }

    #[tokio::test]
    async fn profile_modules_send_the_expected_path_crypto_and_params() {
        for (route, params, want_path, want_data) in profile_cases() {
            let (path, data) = call(route, &params).await;
            assert_eq!(path, want_path, "{route}: 请求路径/加密方式不对");
            assert_eq!(data, want_data, "{route}: 请求参数不对");
        }
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
        for (route, ..) in home_cases().into_iter().chain(profile_cases()) {
            assert!(routes.contains(&route.to_string()), "{route} 没有注册");
        }
    }
}
