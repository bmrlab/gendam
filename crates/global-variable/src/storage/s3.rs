#[macro_export]
macro_rules! s3_storage_new {
    ($library_id:expr, $config:expr) => {{
        use storage::prelude::*;

        S3Storage::new(&$library_id, $config)
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
    ($library_id:expr, $config:expr) => {{
        $crate::get_or_insert_storage!(
            $library_id,
            $crate::s3_storage_new!($library_id, $config),
            $crate::write_s3_storage_map!()
        )
    }};
}

#[macro_export]
macro_rules! get_current_s3_storage {
    ($config:expr) => {{
        let current_library = $crate::current_library!();
        $crate::get_or_insert_s3_storage!(current_library, $config)
    }};
}

#[macro_export]
macro_rules! set_s3_storage {
    ($dir:expr,$storage:expr) => {{
        $crate::set_storage!($crate::S3_STORAGE_MAP, $dir, $storage)
    }};
}
