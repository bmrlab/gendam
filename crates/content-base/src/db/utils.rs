#[macro_export]
macro_rules! collect_async_results {
    ($futures:expr) => {{
        use futures::{stream, StreamExt};

        stream::iter($futures)
            .buffer_unordered(1)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect()
    }};
}

#[macro_export]
macro_rules! concat_arrays {
    ($($arr:expr),+) => {{
        let mut result = Vec::new();
        $(
            result.extend($arr);
        )+
        result.into_boxed_slice()
    }};
}

#[macro_export]
macro_rules! check_db_error_from_resp {
    ($resp:ident) => {{
        let errors_map = $resp.take_errors();
        if !errors_map.is_empty() {
            Err(errors_map)
        } else {
            Ok(())
        }
    }}
}