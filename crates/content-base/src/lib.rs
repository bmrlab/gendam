mod core;
mod db;
pub mod delete;
pub mod query;
pub mod task;
pub mod upsert;

use std::sync::Arc;

pub use content_base_context::ContentBaseCtx;
use content_base_pool::TaskPool;
pub use content_base_pool::{TaskNotification, TaskStatus};
pub use content_base_task::*;
pub use content_metadata::ContentMetadata;
use qdrant_client::Qdrant;

pub mod metadata {
    pub use content_metadata::*;
}

#[derive(Clone)]
pub struct ContentBase {
    ctx: ContentBaseCtx,
    task_pool: TaskPool,
    pub qdrant: Arc<Qdrant>,
    pub language_collection_name: String,
    pub vision_collection_name: String,
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, str::FromStr, sync::Arc, time::Duration};

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
    use global_variable::{init_global_variables, set_current};
    use qdrant_client::Qdrant;

    #[test_log::test(tokio::test)]
    async fn test_task_pool() {
        init_global_variables!();
        // set storage root path
        set_current!("abcdefg".into(), "/Users/zhuo/Desktop".into());

        // the artifacts_dir is relative to the storage root
        let content_base = ContentBaseCtx::new("gendam-test-artifacts", "");

        // initialize AI models
        let whisper =
            AIModel::new(|| async { Whisper::new("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/whisper/ggml-medium-q5_0.bin").await }, None).expect("whisper initialized");
        let llm = AIModel::new(
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
            || async {
                OrtTextEmbedding::new("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/stella-base-zh-v3-1792d/model_quantized.onnx", "/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/stella-base-zh-v3-1792d/tokenizer.json").await
            },
            None,
        ).expect("");
        let tokenizer = Tokenizer::from_file("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/qwen2/tokenizer.json").expect("");

        // add models to ContentBaseCtx
        let content_base = content_base
            .with_audio_transcript(Arc::new(whisper), "whisper")
            .with_llm(Arc::new(llm), tokenizer, "qwen2")
            .with_text_embedding(Arc::new(text_embedding), "stella");

        let file_identifier = "abcdefghijklmn";
        let file_path = PathBuf::from_str("/Users/zhuo/Desktop/测试视频/4月1日.mp4")
            .expect("str should be valid path");

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
        init_global_variables!();
        // set storage root path
        set_current!("abcdefg".into(), "/Users/zhuo/Desktop".into());

        let qdrant = Qdrant::from_url("http://localhost:6334")
            .build()
            .expect("qdrant build");

        // the artifacts_dir is relative to the storage root
        let ctx = ContentBaseCtx::new("gendam-test-artifacts", "");

        // initialize AI models
        let whisper =
            AIModel::new(|| async { Whisper::new("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/whisper/ggml-medium-q5_0.bin").await }, None).expect("whisper initialized");
        let llm = AIModel::new(
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
            || async {
                OrtTextEmbedding::new("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/stella-base-zh-v3-1792d/model_quantized.onnx", "/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/stella-base-zh-v3-1792d/tokenizer.json").await
            },
            None,
        ).expect("");
        let tokenizer = Tokenizer::from_file("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/qwen2/tokenizer.json").expect("");

        // add models to ContentBaseCtx
        let ctx = ctx
            .with_audio_transcript(Arc::new(whisper), "whisper")
            .with_llm(Arc::new(llm), tokenizer, "qwen2")
            .with_text_embedding(Arc::new(text_embedding), "stella");

        let file_identifier = "abcdefghijklmn";
        let file_path = PathBuf::from_str("/Users/zhuo/Desktop/测试视频/4月1日.mp4")
            .expect("str should be valid path");

        let content_base = ContentBase::new(
            &ctx,
            Arc::new(qdrant),
            "content-base-language",
            "content-base-vision",
        )
        .expect("content base created");

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
