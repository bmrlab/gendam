use crate::db::model::id::ID;

#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub id: ID,
    pub distance: f32,
}

pub enum VectorSearchTable {
    Text,
    EnText,
    Image,
    ImagePrompt,
}

impl VectorSearchTable {
    pub fn table_name(&self) -> &str {
        match self {
            VectorSearchTable::Text => "text",
            VectorSearchTable::EnText => "text",
            VectorSearchTable::Image => "image",
            VectorSearchTable::ImagePrompt => "image",
        }
    }

    pub fn column_name(&self) -> &str {
        match self {
            VectorSearchTable::Text => "vector",
            VectorSearchTable::EnText => "en_vector",
            VectorSearchTable::Image => "vector",
            VectorSearchTable::ImagePrompt => "prompt_vector",
        }
    }
}

pub const VECTOR_SEARCH_TABLE: [VectorSearchTable; 4] = [
    VectorSearchTable::Text,
    VectorSearchTable::EnText,
    VectorSearchTable::Image,
    VectorSearchTable::ImagePrompt,
];
