pub const CREATE_TABLE: &str = r#"
-- 创建 "payload" 表
DEFINE TABLE IF NOT EXISTS payload;
-- 定义 "payload" 表的字段
DEFINE FIELD IF NOT EXISTS file_identifier ON TABLE payload TYPE option<string>;
DEFINE FIELD IF NOT EXISTS url ON TABLE payload TYPE option<string>;


-- 创建 "text" 表
DEFINE TABLE IF NOT EXISTS text;
-- 定义 "text" 表的字段
DEFINE FIELD IF NOT EXISTS content ON TABLE text TYPE string;
DEFINE FIELD IF NOT EXISTS embedding ON TABLE text TYPE array;


-- 创建 "image" 表
DEFINE TABLE IF NOT EXISTS image;
-- 定义 "image" 表的字段
-- image vector
DEFINE FIELD IF NOT EXISTS embedding ON TABLE image TYPE array;
DEFINE FIELD IF NOT EXISTS caption ON TABLE image TYPE string;
DEFINE FIELD IF NOT EXISTS caption_embedding ON TABLE image TYPE array;


-- 创建 "image frame" 表
DEFINE TABLE IF NOT EXISTS image_frame;
-- 定义 "image frame" 表的字段
DEFINE FIELD IF NOT EXISTS start_timestamp ON image_frame TYPE number;
DEFINE FIELD IF NOT EXISTS end_timestamp ON image_frame TYPE number;


-- 创建 "audio frame" 表
DEFINE TABLE IF NOT EXISTS audio_frame;
-- 定义 "audio frame" 表的字段
DEFINE FIELD IF NOT EXISTS start_timestamp ON audio_frame TYPE number;
DEFINE FIELD IF NOT EXISTS end_timestamp ON audio_frame TYPE number;


-- 创建 "audio" 表
DEFINE TABLE IF NOT EXISTS audio;
-- 定义 "audio" 表的字段
-- 无，只有 relate 和 with 关系


-- 创建 "video" 表
DEFINE TABLE IF NOT EXISTS video;
-- 定义 "video" 表的字段
-- 无，只有 relate 和 with 关系


-- 创建 "page" 表
DEFINE TABLE IF NOT EXISTS page;
-- 定义 "page" 表的字段
DEFINE FIELD IF NOT EXISTS start_index ON TABLE page TYPE number;
DEFINE FIELD IF NOT EXISTS end_index ON TABLE page TYPE number;


-- 创建 "web_page" 表
DEFINE TABLE IF NOT EXISTS web_page;
-- 定义 "web_page" 表的字段
-- 无，只有 relate 和 with 关系


-- 创建 "document" 表
DEFINE TABLE IF NOT EXISTS document;
-- 定义 "document" 表的字段
-- 无，只有 relate 和 with 关系


-- 定义向量索引
DEFINE INDEX IF NOT EXISTS idx_text_embedding_hnsw_d1024 ON text FIELDS embedding HNSW DIMENSION 1024 DIST EUCLIDEAN;
DEFINE INDEX IF NOT EXISTS idx_image_embedding_hnsw_d512 ON image FIELDS embedding HNSW DIMENSION 512 DIST COSINE;
DEFINE INDEX IF NOT EXISTS idx_image_caption_embedding_hnsw_d1024 ON image FIELDS caption_embedding HNSW DIMENSION 1024 DIST EUCLIDEAN;


-- 定义分词器
-- https://github.com/surrealdb/surrealdb/issues/2850
-- 可定义的分词器
-- https://surrealdb.com/docs/surrealql/statements/define/analyzer
-- 如需高亮，需要指定特定的过滤器
-- https://surrealdb.com/docs/surrealql/functions/database/search#searchhighlight
DEFINE ANALYZER IF NOT EXISTS mixed_analyzer TOKENIZERS blank, class, punct FILTERS lowercase, ascii, snowball(english);

-- 定义索引
DEFINE INDEX IF NOT EXISTS mixed_index_text_content ON text FIELDS content SEARCH ANALYZER mixed_analyzer BM25(1.2, 0.75) HIGHLIGHTS;
DEFINE INDEX IF NOT EXISTS mixed_index_image_caption ON image FIELDS caption SEARCH ANALYZER mixed_analyzer BM25(1.2, 0.75) HIGHLIGHTS;
"#;
