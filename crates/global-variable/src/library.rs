/*
#[macro_export]
macro_rules! set_current_library_dir {
    ($dir:expr) => {{
        let current_library_dir =
            $crate::CURRENT_LIBRARY_DIR.get_or_init(|| $crate::init_current_library_dir!());
        let mut write_current_library_dir = current_library_dir
            .write()
            .expect("Could not write current library dir");
        *write_current_library_dir = $dir;
    }};
}

#[macro_export]
macro_rules! set_current_library {
    ($library_id:expr) => {{
        let current_library =
            $crate::CURRENT_LIBRARY.get_or_init(|| $crate::init_current_library_dir!());
        let mut write_current_library = current_library
            .write()
            .expect("Could not write current library");
        *write_current_library = $library_id;
    }};
}

#[macro_export]
macro_rules! set_current {
    ($library_id:expr, $dir:expr) => {{
        $crate::set_current_library!($library_id);
        $crate::set_current_library_dir!($dir);
    }};
}
*/

#[macro_export]
macro_rules! set_global_current_library {
    ($library_id:expr, $dir:expr) => {{
        // Set CURRENT_LIBRARY global variable
        let current_library =
            $crate::CURRENT_LIBRARY.get_or_init(|| $crate::init_current_library_dir!());
        let mut write_current_library = current_library
            .write()
            .expect("Could not write current library");
        *write_current_library = $library_id;

        // Set CURRENT_LIBRARY_DIR global variable
        let current_library_dir =
            $crate::CURRENT_LIBRARY_DIR.get_or_init(|| $crate::init_current_library_dir!());
        let mut write_current_library_dir = current_library_dir
            .write()
            .expect("Could not write current library dir");
        *write_current_library_dir = $dir;
    }};
}

#[macro_export]
macro_rules! current_library_dir {
    () => {{
        let current_library_dir =
            $crate::CURRENT_LIBRARY_DIR.get_or_init(|| $crate::init_current_library_dir!());
        current_library_dir
            .read()
            .expect("Could not read current library dir")
            .clone()
    }};
}

#[macro_export]
macro_rules! current_library {
    () => {{
        let current_library =
            $crate::CURRENT_LIBRARY.get_or_init(|| $crate::init_current_library_dir!());
        current_library
            .read()
            .expect("Could not read current library")
            .clone()
    }};
}
