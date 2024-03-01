use std::sync::Arc;
use rspc::{
    Rspc,
    BuiltRouter,
};

use crate::routes;
// pub use crate::{R, Ctx};
use crate::CtxWithLibrary;

pub fn get_router<TCtx>() -> Arc<BuiltRouter<TCtx>>
where TCtx: CtxWithLibrary + Clone + Send + Sync + 'static
{
    let router = Rspc::<TCtx>::new().router()
        .merge("users", routes::users::get_routes::<TCtx>())
        .merge("files", routes::files::get_routes::<TCtx>())
        .merge("assets", routes::assets::get_routes::<TCtx>())
        .merge("video", routes::video::get_routes::<TCtx>())
        .merge("radio", routes::radio::get_routes())
        .merge("libraries", routes::library::get_routes::<TCtx>())
        .procedure(
            "version",
            {
                Rspc::<TCtx>::new().query(|_ctx, _input: ()| env!("CARGO_PKG_VERSION"))
            }
        );
    let router = router.build().unwrap();

    #[cfg(debug_assertions)] // Only export in development builds
    {
        use std::path::PathBuf;
        use rspc::ExportConfig;
        router
            .export_ts(ExportConfig::new(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/web/src/lib/bindings.ts"),
            ))
            .unwrap();
    }

    router.arced()
}
