pub(crate) fn contains_invalid_chars(name: &str) -> bool {
    name.chars().any(|c| match c {
        '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => true,
        _ => false,
    })
}

pub(crate) fn normalized_materialized_path(path: &str) -> String {
    if path.ends_with("/") {
        path.to_string()
    } else {
        format!("{}/", path)
    }
}
