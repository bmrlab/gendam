use std::{
    sync::Arc,
    path::PathBuf,
};
use rspc::{
    Rspc,
    BuiltRouter,
    ExportConfig,
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
        router
            .export_ts(ExportConfig::new(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/web/src/lib/bindings.ts"),
            ))
            .unwrap();
    }

    router.arced()
}
