pub const CREATE_TABLE: &str = r#"
-- 创建 "item" 表
DEFINE TABLE IF NOT EXISTS item;
-- 定义 "item" 表的字段
DEFINE FIELD IF NOT EXISTS text ON TABLE item TYPE array<record<text>>;
DEFINE FIELD IF NOT EXISTS image ON TABLE item TYPE array<record<image>>;

-- 创建 "payload" 表
DEFINE TABLE IF NOT EXISTS payload;
-- 定义 "payload" 表的字段
DEFINE FIELD IF NOT EXISTS file_identifier ON TABLE payload TYPE option<string>;
DEFINE FIELD IF NOT EXISTS url ON TABLE payload TYPE option<string>;

---- relation "with" payload
-- 创建 "text" 表
DEFINE TABLE IF NOT EXISTS text;
-- 定义 "text" 表的字段
DEFINE FIELD IF NOT EXISTS data ON TABLE text TYPE string;
DEFINE FIELD IF NOT EXISTS vector ON TABLE text TYPE array;
-- 翻译成英文
DEFINE FIELD IF NOT EXISTS en_data ON TABLE text TYPE string;
DEFINE FIELD IF NOT EXISTS en_vector ON TABLE text TYPE array;

---- relation "with" payload
-- 创建 "image" 表
DEFINE TABLE IF NOT EXISTS image;
-- 定义 "image" 表的字段
-- image vector
DEFINE FIELD IF NOT EXISTS vector ON TABLE image TYPE array;
DEFINE FIELD IF NOT EXISTS prompt ON TABLE image TYPE string;
DEFINE FIELD IF NOT EXISTS prompt_vector ON TABLE image TYPE array;

-- 创建 "image frame" 表
DEFINE TABLE IF NOT EXISTS image_frame;
-- 定义 "image frame" 表的字段
DEFINE FIELD IF NOT EXISTS data ON TABLE image_frame TYPE array<record<image>>;
DEFINE FIELD IF NOT EXISTS start_timestamp ON image_frame TYPE number;
DEFINE FIELD IF NOT EXISTS end_timestamp ON image_frame TYPE number;

-- 创建 "audio frame" 表
DEFINE TABLE IF NOT EXISTS audio_frame;
-- 定义 "audio frame" 表的字段
DEFINE FIELD IF NOT EXISTS data ON TABLE audio_frame TYPE array<record<text>>;
DEFINE FIELD IF NOT EXISTS start_timestamp ON audio_frame TYPE number;
DEFINE FIELD IF NOT EXISTS end_timestamp ON audio_frame TYPE number;

---- relation "with" payload
-- 创建 "audio" 表
DEFINE TABLE IF NOT EXISTS audio;
-- 定义 "audio" 表的字段
DEFINE FIELD IF NOT EXISTS audio_frame ON TABLE audio TYPE array<record<audio_frame>>;

---- relation "with" payload
-- 创建 "video" 表
DEFINE TABLE IF NOT EXISTS video;
-- 定义 "video" 表的字段
DEFINE FIELD IF NOT EXISTS image_frame ON TABLE video TYPE array<record<image_frame>>;
DEFINE FIELD IF NOT EXISTS audio_frame ON TABLE video TYPE array<record<audio_frame>>;

-- 创建 "page" 表
DEFINE TABLE IF NOT EXISTS page;
-- 定义 "page" 表的字段
DEFINE FIELD IF NOT EXISTS text ON TABLE item TYPE array<record<text>>;
DEFINE FIELD IF NOT EXISTS image ON TABLE item TYPE array<record<image>>;
DEFINE FIELD IF NOT EXISTS start_index ON TABLE page TYPE number;
DEFINE FIELD IF NOT EXISTS end_index ON TABLE page TYPE number;

---- relation "with" payload
-- 创建 "web" 表
DEFINE TABLE IF NOT EXISTS web;
-- 定义 "web" 表的字段
DEFINE FIELD IF NOT EXISTS data ON TABLE web TYPE array<record<page>>;

---- relation "with" payload
-- 创建 "document" 表
DEFINE TABLE IF NOT EXISTS document;
-- 定义 "document" 表的字段
DEFINE FIELD IF NOT EXISTS data ON TABLE document TYPE array<record<page>>;

-- 定义向量索引
DEFINE INDEX IF NOT EXISTS idx_text_vector_hnsw_d512 ON text FIELDS vector HNSW DIMENSION 512 DIST EUCLIDEAN;
DEFINE INDEX IF NOT EXISTS idx_image_prompt_vector_hnsw_d512 ON image FIELDS prompt_vector HNSW DIMENSION 512 DIST EUCLIDEAN;
DEFINE INDEX IF NOT EXISTS idx_image_vector_hnsw_d512 ON image FIELDS vector HNSW DIMENSION 512 DIST COSINE;

-- 定义分词器
-- https://github.com/surrealdb/surrealdb/issues/2850
DEFINE ANALYZER IF NOT EXISTS mixed_analyzer TOKENIZERS blank, class, punct FILTERS lowercase, ascii, snowball(english);

-- 定义索引
DEFINE INDEX IF NOT EXISTS mixed_index_text_data ON text FIELDS data SEARCH ANALYZER mixed_analyzer BM25 HIGHLIGHTS;
DEFINE INDEX IF NOT EXISTS mixed_index_text_en_data ON text FIELDS en_data SEARCH ANALYZER mixed_analyzer BM25 HIGHLIGHTS;
DEFINE INDEX IF NOT EXISTS mixed_index_image_prompt ON image FIELDS prompt SEARCH ANALYZER mixed_analyzer BM25 HIGHLIGHTS;
"#;
