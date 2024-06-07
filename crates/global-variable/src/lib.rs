use std::sync::{Arc, OnceLock, RwLock};

pub mod library;
pub mod storage;

pub use ::storage::*;
pub use library::*;
pub use storage::*;

// Map<CURRENT_LIBRARY_DIR, Storage>
pub static STORAGE_MAP: OnceLock<Arc<RwLock<std::collections::HashMap<String, FsStorage>>>> =
    OnceLock::new();

// Map<CURRENT_LIBRARY, Storage>
pub static S3_STORAGE_MAP: OnceLock<Arc<RwLock<std::collections::HashMap<String, S3Storage>>>> =
    OnceLock::new();

pub static CURRENT_LIBRARY_DIR: OnceLock<Arc<RwLock<String>>> = OnceLock::new();

// current library id
pub static CURRENT_LIBRARY: OnceLock<Arc<RwLock<String>>> = OnceLock::new();

#[macro_export]
macro_rules! init_storage_map {
    () => {{
        std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()))
    }};
}

#[macro_export]
macro_rules! init_current_library_dir {
    () => {{
        std::sync::Arc::new(std::sync::RwLock::new("".to_string()))
    }};
}

#[macro_export]
macro_rules! init_global_variables {
    () => {
        $crate::STORAGE_MAP.get_or_init(|| $crate::init_storage_map!());
        $crate::S3_STORAGE_MAP.get_or_init(|| $crate::init_storage_map!());
        $crate::CURRENT_LIBRARY_DIR.get_or_init(|| $crate::init_current_library_dir!());
        $crate::CURRENT_LIBRARY.get_or_init(|| $crate::init_current_library_dir!());
    };
}
