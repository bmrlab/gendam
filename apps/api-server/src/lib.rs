mod ai;
mod content_metadata;
mod cron_jobs;
mod ctx;
mod download;
mod library;
mod routes;
mod standalone;
mod validators;

// re-exports for internal use only
pub(crate) use ctx::traits::CtxWithLibrary;

// Public re-exports for external use only.
// Internal code should use full paths.
pub mod exports {
    pub mod standalone {
        pub use crate::standalone::start_server;
    }
    pub mod storage {
        pub use crate::routes::storage::location::{get_asset_object_location, DataLocationType};
    }
    pub mod ctx {
        pub use crate::ctx::{
            default::Ctx,
            traits::{CtxStore, CtxWithLibrary, StoreError},
        };
    }
    pub use crate::{library::get_library_settings, routes::get_rspc_routes};
}
