use serde::de::{self, Deserializer, Unexpected, Visitor};
use std::fmt;

struct MaterializedPathString;

impl<'de> Visitor<'de> for MaterializedPathString {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a materialized path string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value == "/" {
            return Ok(value.to_string());
        }
        // check value is matches regexpr /^(\/[^/\\:*?"<>|]+)+\/$/
        let re = regex::Regex::new(r#"^(\/[^/\\:*?"<>|]+)+\/$"#).unwrap();
        if re.is_match(value) {
            Ok(value.to_string())
        } else {
            Err(de::Error::invalid_value(Unexpected::Str(value), &self))
        }
    }
}

pub fn materialized_path_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(MaterializedPathString)
}

struct PathNameString;

impl<'de> Visitor<'de> for PathNameString {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a path name string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value.chars().any(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => true,
            _ => false,
        }) {
            Err(de::Error::invalid_value(Unexpected::Str(value), &self))
        } else {
            Ok(value.to_string())
        }
    }
}

pub fn path_name_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(PathNameString)
}

pub fn replace_invalid_chars_in_path_name(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => ' ',
            _ => c,
        })
        .collect()
}
