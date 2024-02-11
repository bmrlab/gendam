use std::{
    sync::Arc,
    path::PathBuf,
};
use rspc::{
    BuiltRouter,
    ExportConfig,
};

use crate::routes;
pub use crate::{R, Ctx};

pub fn get_router() -> Arc<BuiltRouter<Ctx>> {
    let router = R.router()
        .merge("users", routes::users::get_routes())
        .merge("files", routes::files::get_routes())
        .procedure(
            "version",
            R.query(|_ctx, _input: ()| env!("CARGO_PKG_VERSION"))
        );
    let router = router.build().unwrap().arced();
    router
        .export_ts(ExportConfig::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/web/src/lib/bindings.ts"),
        ))
        .unwrap();
    return router;
}
