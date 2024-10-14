use crate::constant::HIGHLIGHT_MARK;
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
macro_rules! collect_ordered_async_results {
    ($futures:expr, $ty:ty) => {{
        use futures::{stream, StreamExt};

        stream::iter($futures)
            .buffered(1)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(Result::ok)
            .collect::<$ty>()
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

#[allow(dead_code)]
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

pub fn extract_highlighted_content(text: &str) -> Vec<String> {
    Regex::new(&format!(r"{}(.*?){}", HIGHLIGHT_MARK.0, HIGHLIGHT_MARK.1))
        .and_then(|re| {
            Ok(re
                .captures_iter(text)
                .filter_map(|cap| cap.get(1))
                .map(|m| m.as_str().to_string())
                .collect::<Vec<String>>())
        })
        .unwrap_or(vec![])
}

#[cfg(test)]
mod test {
    use crate::utils::extract_highlighted_content;

    #[test]
    fn test_escape_single_quotes() {
        assert_eq!(super::escape_single_quotes("I'm"), "I\\'m");
    }

    #[test]
    fn test_extract_highlighted_content() {
        let text = "这里是一些文本<b>需要提取的内容1</b>和一些其他文本<b>需要提取的内容2</b>。";
        let result = extract_highlighted_content(text);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "需要提取的内容1");
        assert_eq!(result[1], "需要提取的内容2");
    }

    #[test]
    fn test_no_highlighted_content() {
        let text = "这里没有任何高亮内容。";
        let result = extract_highlighted_content(text);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_partial_highlighted_content() {
        let text = "<b>只提取这部分</b>和其他文本<b>再提取这部分</b>。";
        let result = extract_highlighted_content(text);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "只提取这部分");
        assert_eq!(result[1], "再提取这部分");
    }
}
