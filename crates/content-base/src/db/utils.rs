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