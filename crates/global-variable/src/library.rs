#[macro_export]
macro_rules! read_current_library_dir {
    () => {{
        let current_library_dir =
            $crate::CURRENT_LIBRARY_DIR.get_or_init(|| $crate::init_current_library_dir!());
        current_library_dir.read().map_err(|e| {
            StorageError::MutexPoisonError("Fail to read current library dir".to_string())
        })
    }};
}

#[macro_export]
macro_rules! write_current_library_dir {
    () => {{
        let current_library_dir =
            $crate::CURRENT_LIBRARY_DIR.get_or_init(|| $crate::init_current_library_dir!());
        current_library_dir
            .write()
            .map_err(|e| anyhow::anyhow!("Could not write current library dir: {e}"))
    }};
}

#[macro_export]
macro_rules! set_current_library_dir {
    ($dir:expr) => {{
        match $crate::write_current_library_dir!() {
            Ok(mut current_library_dir) => {
                *current_library_dir = $dir;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }};
}

#[macro_export]
macro_rules! current_library_dir {
    () => {{
        $crate::read_current_library_dir!().unwrap().clone()
    }};
}
