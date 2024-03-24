use crate::routes;
use rspc::Router;
use crate::CtxWithLibrary;

pub fn get_router<TCtx>() -> Router<TCtx>
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
        .merge("users.", routes::users::get_routes::<TCtx>())
        .merge("files.", routes::files::get_routes::<TCtx>())
        .merge("assets.", routes::assets::get_routes::<TCtx>())
        .merge("video.", routes::video::get_routes::<TCtx>())
        .merge("audio.", routes::audio::get_routes::<TCtx>())
        .merge("libraries.", routes::library::get_routes::<TCtx>())
        .query("version", |t| {
            t(|_ctx, _input: ()| env!("CARGO_PKG_VERSION"))
        })
        .build();

    #[cfg(debug_assertions)] // Only export in development builds
    {
        use std::path::PathBuf;
        if let Err(e) = router.export_ts(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/web/src/lib/bindings.ts"),
        ) {
            tracing::error!("Failed to export typescript bindings: {}", e);
        }
    }

    router
}
