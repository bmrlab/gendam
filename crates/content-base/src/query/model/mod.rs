use full_text::FullTextSearchResult;
use vector::VectorSearchResult;

pub mod full_text;
pub mod vector;

pub struct TextToken(pub Vec<String>);

pub struct TextSearchModel {
    pub data: String,
    pub tokens: TextToken,
    /// 用于查询文本向量
    pub text_vector: Vec<f32>,
    /// 用于查询图像向量
    pub vision_vector: Vec<f32>,
}

pub struct ImageSearchModel {
    pub prompt: String,
    pub prompt_search_model: TextSearchModel,
    /// 用于查询文本向量
    pub text_vector: Vec<f32>,
    /// 用于查询图像向量
    pub vision_vector: Vec<f32>,
}

pub enum SearchModel {
    Text(TextSearchModel),
    Image(ImageSearchModel),
    // TODO: 其他类型待补充
}

pub enum SearchResult {
    Vector(VectorSearchResult),
    FullText(FullTextSearchResult),
}
