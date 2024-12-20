mod constant;
mod core;
pub mod db;
pub mod delete;
pub mod query;
pub mod task;
pub mod upsert;
mod utils;

use std::sync::Arc;

use crate::db::DB;
pub use content_base_context::ContentBaseCtx;
use content_base_pool::TaskPool;
pub use content_base_pool::{TaskNotification, TaskStatus};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ContentBase {
    ctx: ContentBaseCtx,
    task_pool: TaskPool,
    surrealdb_client: Arc<RwLock<DB>>,
}

#[cfg(test)]
mod test {
    use crate::db::shared::test::setup;
    use crate::{upsert::UpsertPayload, ContentBase};
    use ai::{
        llm::{openai::OpenAI, LLM},
        text_embedding::OrtTextEmbedding,
        tokenizers::Tokenizer,
        whisper::Whisper,
        AIModel,
    };
    use content_base_context::ContentBaseCtx;
    use content_base_pool::TaskPool;
    use content_base_task::{
        video::{
            audio::VideoAudioTask, frame::VideoFrameTask, thumbnail::VideoThumbnailTask,
            trans_chunk::VideoTransChunkTask, trans_chunk_sum::VideoTransChunkSumTask,
            trans_chunk_sum_embed::VideoTransChunkSumEmbedTask, transcript::VideoTranscriptTask,
        },
        ContentTaskType, TaskRecord,
    };
    use content_handler::{file_metadata, video::VideoDecoder};
    use content_metadata::ContentMetadata;
    use std::path::Path;
    use std::{env, path::PathBuf, str::FromStr, sync::Arc, time::Duration};
    use tokio::sync::RwLock;

    fn get_project_root() -> PathBuf {
        let mut path = Path::new(env!("CARGO_MANIFEST_DIR"));

        while let Some(parent) = path.parent() {
            if parent.join("Cargo.toml").exists() {
                return parent.to_path_buf();
            }
            path = parent;
        }
        path.to_path_buf()
    }

    fn get_desktop_path() -> PathBuf {
        let home_dir = env::var("HOME").expect("HOME env var should be set");
        PathBuf::from(home_dir).join("Desktop")
    }

    #[test_log::test(tokio::test)]
    async fn test_task_pool() {
        global_variable::init_global_variables!();
        // set storage root path
        global_variable::set_global_current_library!(
            "abcdefg".into(),
            get_desktop_path().to_str().unwrap().into()
        );
        // the artifacts_dir is relative to the storage root
        let content_base = ContentBaseCtx::new("gendam-test-artifacts", "");

        // initialize AI models
        let whisper = AIModel::new(
            "whisper-small".into(),
            || async {
                Whisper::new(
                    get_project_root()
                        .join("apps/desktop/src-tauri/resources/whisper/ggml-small-q5_1.bin"),
                )
                .await
            },
            None,
        )
        .expect("whisper initialized");
        let llm = AIModel::new(
            "ollama-qwen2-7b-instruct".into(),
            || async {
                Ok(LLM::OpenAI(
                    OpenAI::new(
                        "http://localhost:11434/v1",
                        "ollama",
                        "qwen2:7b-instruct-q4_0",
                    )
                    .expect(""),
                ))
            },
            None,
        )
        .expect("");
        let text_embedding = AIModel::new(
            "puff-base-v1".into(),
            || async {
                OrtTextEmbedding::new(
                    get_project_root()
                        .join("apps/desktop/src-tauri/resources/puff-base-v1/model_quantized.onnx"),
                    get_project_root()
                        .join("apps/desktop/src-tauri/resources/puff-base-v1/tokenizer.json"),
                )
                .await
            },
            None,
        )
        .expect("");
        let tokenizer = Tokenizer::from_file(
            get_project_root().join("apps/desktop/src-tauri/resources/qwen2/tokenizer.json"),
        )
        .expect("");

        // add models to ContentBaseCtx
        let content_base = content_base
            .with_audio_transcript(Arc::new(whisper), "whisper")
            .with_llm(Arc::new(llm), "qwen2")
            .with_text_tokenizer(Arc::new(tokenizer), "qwen2")
            .with_text_embedding(Arc::new(text_embedding), "puff");

        let file_identifier = "abcdefghijklmn";
        let file_path = get_desktop_path().join("测试视频/4月1日.mp4");

        let video_decoder = VideoDecoder::new(&file_path).expect("video decoder built");
        let metadata = video_decoder.get_video_metadata().expect("got metadata");
        let metadata = ContentMetadata::Video(metadata);

        let mut task_record = TaskRecord::from_content_base(file_identifier, &content_base).await;
        task_record
            .set_metadata(&content_base, &metadata)
            .await
            .expect("set metadata");

        tracing::info!("metadata: {:?}", metadata);

        // init task pool
        let task_pool = TaskPool::new(&content_base, None).expect("task pool created");

        let tasks: Vec<ContentTaskType> = vec![
            VideoThumbnailTask.into(),
            VideoFrameTask.into(),
            VideoAudioTask.into(),
            VideoTranscriptTask.into(),
            VideoTransChunkTask.into(),
            VideoTransChunkSumTask.into(),
            VideoTransChunkSumEmbedTask.into(),
        ];

        for task in tasks.iter() {
            let result = task_pool
                .add_task(&file_identifier, &file_path, task, None, None)
                .await;
            tracing::info!("task insert result: {:?}", result);
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }

    #[test_log::test(tokio::test(flavor = "multi_thread"))]
    async fn test_core() {
        global_variable::init_global_variables!();
        // set storage root path
        global_variable::set_global_current_library!(
            "abcdefg".into(),
            "/Users/zhuo/Desktop".into()
        );

        // the artifacts_dir is relative to the storage root
        let ctx = ContentBaseCtx::new("gendam-test-artifacts", "");

        // initialize AI models
        let whisper = AIModel::new(
            "whisper-small".into(),
            || async {
                Whisper::new("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/whisper/ggml-medium-q5_0.bin").await
            },
            None
        )
        .expect("whisper initialized");
        let llm = AIModel::new(
            "ollama-qwen2-7b-instruct".into(),
            || async {
                Ok(LLM::OpenAI(
                    OpenAI::new(
                        "http://localhost:11434/v1",
                        "ollama",
                        "qwen2:7b-instruct-q4_0",
                    )
                    .expect(""),
                ))
            },
            None,
        )
        .expect("");
        let text_embedding = AIModel::new(
            "stella-base-zh-v3-1792d".into(),
            || async {
                OrtTextEmbedding::new("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/stella-base-zh-v3-1792d/model_quantized.onnx", "/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/stella-base-zh-v3-1792d/tokenizer.json").await
            },
            None,
        ).expect("");
        let tokenizer = Tokenizer::from_file("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/qwen2/tokenizer.json").expect("");

        // add models to ContentBaseCtx
        let ctx = ctx
            .with_audio_transcript(Arc::new(whisper), "whisper")
            .with_llm(Arc::new(llm), "qwen2")
            .with_text_tokenizer(Arc::new(tokenizer), "qwen2")
            .with_text_embedding(Arc::new(text_embedding), "stella");

        let file_identifier = "abcdefghijklmn";
        let file_path = PathBuf::from_str("/Users/zhuo/Desktop/测试视频/4月1日.mp4")
            .expect("str should be valid path");

        let db = setup(Some(env::current_exe().unwrap().parent().unwrap())).await;

        let content_base =
            ContentBase::new(&ctx, Arc::new(RwLock::new(db))).expect("content base created");

        let (metadata, _) = file_metadata(&file_path, Some("mp4"));

        let upsert_result = content_base
            .upsert(UpsertPayload::new(
                file_identifier.into(),
                file_path,
                &metadata,
            ))
            .await;

        tracing::info!("upsert result: {:?}", upsert_result);

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
