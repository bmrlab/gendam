pub mod fs;
pub mod s3;

#[macro_export]
macro_rules! read_storage_map {
    ($map:expr) => {{
        $map.get_or_init(|| $crate::init_storage_map!())
            .read()
            .map_err(|_| StorageError::MutexPoisonError("Fail to read storage map".to_string()))
    }};
}

#[macro_export]
macro_rules! write_storage_map {
    ($map:expr) => {{
        $map.get_or_init(|| $crate::init_storage_map!())
            .write()
            .map_err(|_| StorageError::MutexPoisonError("Fail to write storage map".to_string()))
    }};
}

#[macro_export]
macro_rules! get_or_insert_storage {
    ($root_path:expr, $storage:expr, $write_map:expr) => {{
        use storage::prelude::*;

        match $write_map {
            std::result::Result::Ok(mut map) => std::result::Result::Ok(
                map.entry($root_path.clone())
                    .or_insert_with(|| $storage)
                    .clone(),
            ),
            std::result::Result::Err(e) => std::result::Result::Err(e),
        }
    }};
}

#[macro_export]
macro_rules! set_storage {
    ($map:expr, $dir:expr, $storage:expr) => {{
        match $crate::write_storage_map!($map:expr) {
            std::result::Result::Ok(mut map) => {
                map.insert($dir, $storage);
                std::result::Result::Ok(())
            }
            std::result::Result::Err(e) => std::result::Result::Err(e),
        }
    }};
}
