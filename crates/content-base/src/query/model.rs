use crate::db::model::id::ID;

pub struct TextToken(pub Vec<String>);

pub struct TextSearchModel {
    pub data: String,
    pub tokens: TextToken,
    /// 用于查询文本向量
    pub text_embedding: Vec<f32>,
    /// 用于查询图像向量
    pub vision_embedding: Vec<f32>,
}

pub struct ImageSearchModel {
    pub prompt: String,
    pub prompt_search_model: TextSearchModel,
    /// 用于查询文本向量
    pub text_embedding: Vec<f32>,
    /// 用于查询图像向量
    pub vision_embedding: Vec<f32>,
}

pub enum SearchModel {
    Text(TextSearchModel),
    Image(ImageSearchModel),
    // TODO: 其他类型待补充
}

#[derive(Clone, Debug, PartialEq)]
pub enum SearchType {
    Vector(VectorSearchType),
    FullText,
}

// use full_text::FullTextSearchResult;
// use vector::VectorSearchResult;
// pub enum SearchResult {
//     Vector(VectorSearchResult),
//     FullText(FullTextSearchResult),
// }

#[derive(Debug, Clone, PartialEq)]
pub enum VectorSearchType {
    Text,   // 搜索文本向量
    Vision, // 搜索图像向量
}

#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub id: ID,
    pub distance: f32,
    pub vector_type: VectorSearchType,
}

#[derive(Debug, Clone)]
pub struct FullTextSearchResult {
    pub id: ID,
    /// 分词，分数
    /// 如果是 with highlight，则 score 只有一个元素
    pub score: Vec<(String, f32)>,
}
