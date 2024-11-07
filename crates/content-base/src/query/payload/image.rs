use serde::Serialize;

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct ImageIndexMetadata {
    pub data: i32, // 这个值没有意义，只是为了 rspc 可以正常的 serialize 这个对象
}
