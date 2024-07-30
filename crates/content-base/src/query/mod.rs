pub mod payload;
pub mod rag;
pub mod search;

use crate::ContentBase;
use rag::RAGReference;
pub use search::SearchResult;

pub struct QueryPayload {
    query: String,
}

impl QueryPayload {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
        }
    }
}

pub struct RecommendPayload {
    file_identifier: String,
}

impl RecommendPayload {
    pub fn new(file_identifier: &str) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
        }
    }
}


impl RAGPayload {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
        }
    }
}

pub struct RAGPayload {
    query: String,
}

impl ContentBase {
    pub async fn query(&self, payload: QueryPayload) -> anyhow::Result<Vec<SearchResult>> {
        todo!()
    }

    pub async fn recommend(&self, payload: RecommendPayload) -> anyhow::Result<Vec<SearchResult>> {
        todo!()
    }

    pub async fn retrieval(&self, payload: RAGPayload) -> anyhow::Result<Vec<RAGReference>> {
        todo!()
    }
}
