mod audio;
mod libraries;
mod search;
mod tasks;
mod users;
mod video;

pub(crate) mod assets;
pub(crate) mod localhost;
pub(crate) mod p2p;
pub(crate) mod storage;

use crate::CtxWithLibrary;
use rspc::Router;

pub fn get_rspc_routes<TCtx>() -> Router<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    let router = Router::<TCtx>::new()
        // .middleware(|mw| {
        //     mw.middleware(|mw| async move {
        //         // let local_data_root = ctx.local_data_root;
        //         let ctx = mw.ctx;
        //         Ok(mw.with_ctx(ctx))
        //     })
        // })
        .merge("users.", users::get_routes::<TCtx>())
        .merge("assets.", assets::get_routes::<TCtx>())
        .merge("tasks.", tasks::get_routes::<TCtx>())
        .merge("search.", search::get_routes::<TCtx>())
        .merge("video.", video::get_routes::<TCtx>())
        .merge("audio.", audio::get_routes::<TCtx>())
        .merge("libraries.", libraries::get_routes::<TCtx>())
        .merge("p2p.", p2p::get_routes::<TCtx>())
        .merge("storage.", storage::get_routes::<TCtx>())
        .query("version", |t| {
            t(|_ctx, _input: ()| env!("CARGO_PKG_VERSION"))
        })
        .build();

    #[cfg(debug_assertions)] // Only export in development builds
    {
        use std::path::PathBuf;
        if let Err(e) =
            router.export_ts(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("client/types.ts"))
        {
            tracing::error!("Failed to export typescript bindings: {}", e);
        }
    }

    router
}
