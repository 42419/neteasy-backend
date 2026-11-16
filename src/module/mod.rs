//! Mirrors api-enhanced `module/`: one file per endpoint. File `song_url_v1.rs` serves `/song/url/v1`
//! (route = file name with `_` replaced by `/`, exactly like upstream `getModulesDefinitions`).
//!
//! Only the endpoints the neteasy app uses are ported; see README for the status table.

use crate::util::index::Query;
use crate::util::request::{AnswerResult, Request};
use std::future::Future;
use std::pin::Pin;

pub type ModuleFuture<'a> = Pin<Box<dyn Future<Output = AnswerResult> + Send + 'a>>;
/// Upstream module contract: `(query, request) => Promise<{ status, body, cookie }>`.
pub type ModuleFn = for<'a> fn(Query, &'a Request) -> ModuleFuture<'a>;

macro_rules! modules {
    ($($name:ident),* $(,)?) => {
        $(pub mod $name;)*

        /// `(identifier, route, handler)` for every ported module.
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
