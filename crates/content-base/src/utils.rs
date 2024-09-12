use regex::Regex;
use std::collections::HashSet;

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

pub fn escape_single_quotes(input: &str) -> String {
    let re = Regex::new(r"'").unwrap();
    re.replace_all(input, "\\'").to_string()
}

// 字符串数组去重
pub fn deduplicate(vec: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for item in vec {
        if seen.insert(item.clone()) {
            deduped.push(item);
        }
    }
    deduped
}

#[cfg(test)]
mod test {
    #[test]
    fn test_escape_single_quotes() {
        assert_eq!(super::escape_single_quotes("I'm"), "I\\'m");
    }
}
