use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use storage::Storage;

// Map<root_path, Storage>
pub static STORAGE_MAP: OnceLock<Arc<RwLock<HashMap<String, Storage>>>> = OnceLock::new();

pub static CURRENT_LIBRARY_DIR: OnceLock<Arc<RwLock<String>>> = OnceLock::new();

pub fn init_storage_map() -> Arc<RwLock<HashMap<String, Storage>>> {
    Arc::new(RwLock::new(HashMap::new()))
}

pub fn init_current_library_dir() -> Arc<RwLock<String>> {
    Arc::new(RwLock::new("".to_string()))
}

#[macro_export]
macro_rules! init_global_variables {
    () => {
        $crate::global::STORAGE_MAP.get_or_init(|| $crate::global::init_storage_map());
        $crate::global::CURRENT_LIBRARY_DIR
            .get_or_init(|| $crate::global::init_current_library_dir());
    };
}

#[macro_export]
macro_rules! read_storage_map {
    () => {{
        STORAGE_MAP
            .get_or_init(|| init_storage_map())
            .read()
            .map_err(|e| anyhow::anyhow!("Could not read storage map: {e}"))
    }};
}

#[macro_export]
macro_rules! write_storage_map {
    () => {{
        STORAGE_MAP
            .get_or_init(|| init_storage_map())
            .write()
            .map_err(|e| anyhow::anyhow!("Could not write storage map: {e}"))
    }};
}

#[macro_export]
macro_rules! read_current_library_dir {
    () => {{
        let current_library_dir = CURRENT_LIBRARY_DIR.get_or_init(|| init_current_library_dir());
        current_library_dir
            .read()
            .map_err(|e| anyhow::anyhow!("Could not read current library dir: {e}"))
    }};
}

#[macro_export]
macro_rules! write_current_library_dir {
    () => {{
        let current_library_dir = CURRENT_LIBRARY_DIR.get_or_init(|| init_current_library_dir());
        current_library_dir
            .write()
            .map_err(|e| anyhow::anyhow!("Could not write current library dir: {e}"))
    }};
}

#[macro_export]
macro_rules! get_current_storage {
    () => {{
        let current_library_dir = read_current_library_dir!()?.clone();
        let map = read_storage_map!()?;
        map.get(&current_library_dir)
            .map(|s| s.clone())
            .ok_or_else(|| anyhow::anyhow!("No storage found for {current_library_dir}"))
    }};
}

#[macro_export]
macro_rules! set_storage {
    (library_dir = $dir:expr, storage = $storage:expr) => {{
        let map = STORAGE_MAP.get_or_init(|| init_storage_map());
        let mut write_guard = map
            .write()
            .map_err(|e| anyhow::anyhow!("Could not write storage map: {e}"))?;
        write_guard.insert(library_dir, storage);
        Ok(())
    }};
}

#[macro_export]
macro_rules! set_current_library_dir {
    ($dir:expr) => {{
        let current_library_dir = CURRENT_LIBRARY_DIR.get_or_init(|| init_current_library_dir());
        *write_current_library_dir!()? = $dir;
        Ok(())
    }};
}
