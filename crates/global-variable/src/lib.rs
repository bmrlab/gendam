use std::sync::{Arc, OnceLock, RwLock};

use ::storage::Storage;

pub mod library;
pub mod storage;

// Map<root_path, Storage>
pub static STORAGE_MAP: OnceLock<Arc<RwLock<std::collections::HashMap<String, Storage>>>> =
    OnceLock::new();

pub static CURRENT_LIBRARY_DIR: OnceLock<Arc<RwLock<String>>> = OnceLock::new();

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
        $crate::CURRENT_LIBRARY_DIR.get_or_init(|| $crate::init_current_library_dir!());
    };
}
