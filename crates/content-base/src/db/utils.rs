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
