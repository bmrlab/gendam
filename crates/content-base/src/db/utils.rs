#[macro_export]
macro_rules! check_db_error_from_resp {
    ($resp:ident) => {{
        let errors_map = $resp.take_errors();
        if !errors_map.is_empty() {
            Err(errors_map)
        } else {
            Ok(())
        }
    }};
}
