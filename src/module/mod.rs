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
    like,
    likelist,
    login_qr_check,
    login_qr_create,
    login_qr_key,
    login_status,
    lyric,
    lyric_new,
    personalized,
    playlist_detail,
    playlist_track_all,
    recommend_songs,
    register_anonimous,
    register_xeapikey,
    search,
    search_hot,
    search_suggest,
    song_detail,
    song_url_v1,
    toplist,
    user_detail,
    user_playlist,
}
