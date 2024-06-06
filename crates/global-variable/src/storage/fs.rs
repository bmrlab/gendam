#[macro_export]
macro_rules! fs_storage_new {
    ($root_path:expr) => {{
        use storage::FsStorage;
        use storage::StorageError;

        FsStorage::new(&$root_path)
            .map_err(|e| StorageError::UnexpectedError)
            .unwrap()
    }};
}

#[macro_export]
macro_rules! read_fs_storage_map {
    () => {{
        $crate::read_storage_map!($crate::STORAGE_MAP)
    }};
}

#[macro_export]
macro_rules! write_fs_storage_map {
    () => {{
        $crate::write_storage_map!($crate::STORAGE_MAP)
    }};
}

#[macro_export]
macro_rules! get_or_insert_fs_storage {
    ($root_path:expr) => {{
        $crate::get_or_insert_storage!($root_path, $crate::fs_storage_new!($root_path))
    }};
}

#[macro_export]
macro_rules! get_current_fs_storage {
    () => {{
        use storage::StorageError;

        $crate::get_current_storage!($crate::get_or_insert_fs_storage!(
            $crate::current_library_dir!()
        ))
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
