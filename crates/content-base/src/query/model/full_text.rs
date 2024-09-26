use crate::db::model::id::ID;

#[derive(Debug, Clone)]
pub struct FullTextSearchResult {
    pub id: ID,
    /// 分词，分数
    /// 如果是 with highlight，则 score 只有一个元素
    pub score: Vec<(String, f32)>,
}

pub enum FullTextSearchTable {
    Text,
    EnText,
    Image,
}

impl FullTextSearchTable {
    pub fn table_name(&self) -> &str {
        match self {
            FullTextSearchTable::Text => "text",
            FullTextSearchTable::EnText => "text",
            FullTextSearchTable::Image => "image",
        }
    }

    pub fn column_name(&self) -> &str {
        match self {
            FullTextSearchTable::Text => "data",
            FullTextSearchTable::EnText => "en_data",
            FullTextSearchTable::Image => "prompt",
        }
    }
}

pub const FULL_TEXT_SEARCH_TABLE: [FullTextSearchTable; 3] = [
    FullTextSearchTable::Text,
    FullTextSearchTable::EnText,
    FullTextSearchTable::Image,
];
