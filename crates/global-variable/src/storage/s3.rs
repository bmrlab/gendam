#[macro_export]
macro_rules! s3_storage_new {
    ($root_path:expr) => {{
        use storage::prelude::*;

        S3Storage::new(&$root_path)
            .map_err(|e| StorageError::UnexpectedError)
            .unwrap()
    }};
}

#[macro_export]
macro_rules! read_s3_storage_map {
    () => {{
        $crate::read_storage_map!($crate::S3_STORAGE_MAP)
    }};
}

#[macro_export]
macro_rules! write_s3_storage_map {
    () => {{
        $crate::write_storage_map!($crate::S3_STORAGE_MAP)
    }};
}

#[macro_export]
macro_rules! get_or_insert_s3_storage {
    ($root_path:expr) => {{
        $crate::get_or_insert_storage!($root_path, $crate::s3_storage_new!($root_path))
    }};
}

#[macro_export]
macro_rules! get_current_s3_storage {
    () => {{
        use storage::StorageError;

        $crate::get_current_storage!($crate::get_or_insert_s3_storage!(
            $crate::current_library_dir!()
        ))
    }};
}

#[macro_export]
macro_rules! set_s3_storage {
    ($dir:expr,$storage:expr) => {{
        $crate::set_storage!($crate::S3_STORAGE_MAP, $dir, $storage)
    }};
}
