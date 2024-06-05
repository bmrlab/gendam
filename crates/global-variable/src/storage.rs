#[macro_export]
macro_rules! read_storage_map {
    () => {{
        $crate::STORAGE_MAP
            .get_or_init(|| $crate::init_storage_map!())
            .read()
            .map_err(|_| StorageError::MutexPoisonError("Fail to read storage map".to_string()))
    }};
}

#[macro_export]
macro_rules! write_storage_map {
    () => {{
        $crate::STORAGE_MAP
            .get_or_init(|| $crate::init_storage_map!())
            .write()
            .map_err(|_| StorageError::MutexPoisonError("Fail to write storage map".to_string()))
    }};
}

#[macro_export]
macro_rules! get_or_insert_storage {
    ($root_path:expr) => {{
        use storage::FsStorage;
        use storage::StorageError;

        match $crate::write_storage_map!() {
            std::result::Result::Ok(mut map) => std::result::Result::Ok(
                map.entry($root_path.clone())
                    .or_insert_with(|| {
                        FsStorage::new(&$root_path)
                            .map_err(|e| StorageError::UnexpectedError)
                            .unwrap()
                    })
                    .clone(),
            ),
            std::result::Result::Err(e) => std::result::Result::Err(e),
        }
    }};
}

#[macro_export]
macro_rules! get_current_storage {
    () => {{
        match $crate::read_current_library_dir!() {
            std::result::Result::Ok(current_library_dir) => {
                let current_library_dir = current_library_dir.clone();
                $crate::get_or_insert_storage!(current_library_dir)
            }
            std::result::Result::Err(e) => std::result::Result::Err(e),
        }
    }};
}

#[macro_export]
macro_rules! set_storage {
    (library_dir = $dir:expr, storage = $storage:expr) => {{
        let map = $crate::STORAGE_MAP.get_or_init(|| init_storage_map!());
        let mut write_guard = map
            .write()
            .map_err(|e| anyhow::anyhow!("Could not write storage map: {e}"))?;
        write_guard.insert(library_dir, storage);
        Ok(())
    }};
}
