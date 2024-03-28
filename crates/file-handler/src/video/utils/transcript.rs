use super::save_text_embedding;
use crate::{
    search_payload::SearchPayload,
    video::{AUDIO_FILE_NAME, TRANSCRIPT_FILE_NAME},
};
use ai::{
    clip::CLIP,
    whisper::{Whisper, WhisperItem, WhisperParams},
    BatchHandler,
};
use prisma_lib::{video_transcript, PrismaClient};
use qdrant_client::client::QdrantClient;
use std::{fs::File, io::BufReader, path::Path, sync::Arc};
use tokio::io::AsyncWriteExt;
use tracing::error;

pub async fn save_transcript(
    artifacts_dir: impl AsRef<std::path::Path>,
    file_identifier: String,
    client: Arc<PrismaClient>,
    whisper: BatchHandler<Whisper>,
) -> anyhow::Result<()> {
    let result = whisper
        .process_single((
            artifacts_dir.as_ref().join(AUDIO_FILE_NAME),
            Some(WhisperParams {
                enable_translate: false,
                ..Default::default()
            }),
        ))
        .await?;

    // write results into json file
    let mut file =
        tokio::fs::File::create(artifacts_dir.as_ref().join(TRANSCRIPT_FILE_NAME)).await?;
    let json = serde_json::to_string(&result.items())?;
    file.write_all(json.as_bytes()).await?;

    let mut join_set = tokio::task::JoinSet::new();

    for item in result.items() {
        let file_identifier = file_identifier.clone();
        let client = client.clone();

        join_set.spawn(async move {
            let x = {
                client
                    .video_transcript()
                    .upsert(
                        video_transcript::file_identifier_start_timestamp_end_timestamp(
                            file_identifier.clone(),
                            item.start_timestamp as i32,
                            item.end_timestamp as i32,
                        ),
                        (
                            file_identifier.clone(),
                            item.start_timestamp as i32,
                            item.end_timestamp as i32,
                            item.text.clone(), // store original text
                            vec![],
                        ),
                        vec![],
                    )
                    .exec()
                    .await
            };

            if let Err(e) = x {
                error!("failed to save transcript: {:?}", e);
            }
        });
    }

    while let Some(_) = join_set.join_next().await {}

    Ok(())
}

#[deprecated(note = "this function need to be improved")]
#[allow(dead_code)]
pub async fn save_transcript_embedding(
    file_identifier: String,
    client: Arc<PrismaClient>,
    path: impl AsRef<Path>,
    clip_model: BatchHandler<CLIP>,
    qdrant: Arc<QdrantClient>,
) -> anyhow::Result<()> {
    let whisper_results: Vec<WhisperItem> = {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        // Read the JSON contents of the file as an instance of `WhisperItem`
        serde_json::from_reader(reader)?
    };

    let clip_model = clip_model.clone();

    let mut join_set = tokio::task::JoinSet::new();

    for item in whisper_results {
        // if item is some like [MUSIC], just skip it
        // TODO need to make sure all filter rules
        if item.text.starts_with("[") || item.text.starts_with("(") {
            continue;
        }

        let clip_model = clip_model.clone();
        let file_identifier = file_identifier.clone();
        let client = client.clone();
        let qdrant = qdrant.clone();

        join_set.spawn(async move {
            // write data using prisma
            // here use write to make sure only one thread can using prisma client
            let x = {
                client
                    .video_transcript()
                    .upsert(
                        video_transcript::file_identifier_start_timestamp_end_timestamp(
                            file_identifier.clone(),
                            item.start_timestamp as i32,
                            item.end_timestamp as i32,
                        ),
                        (
                            file_identifier.clone(),
                            item.start_timestamp as i32,
                            item.end_timestamp as i32,
                            item.text.clone(), // store original text
                            vec![],
                        ),
                        vec![],
                    )
                    .exec()
                    .await
                // drop the rwlock
            };

            match x {
                std::result::Result::Ok(res) => {
                    let payload = SearchPayload::Transcript {
                        id: res.id as u64,
                        file_identifier: file_identifier.clone(),
                        start_timestamp: item.start_timestamp,
                        end_timestamp: item.end_timestamp,
                    };
                    if let Err(e) = save_text_embedding(
                        &item.text, // but embedding english
                        payload,
                        clip_model,
                        qdrant,
                        vector_db::DEFAULT_COLLECTION_NAME,
                    )
                    .await
                    {
                        error!("failed to save transcript embedding: {:?}", e);
                    }
                }
                Err(e) => {
                    error!("failed to save transcript embedding: {:?}", e);
                }
            }
        });
    }

    while let Some(_) = join_set.join_next().await {}

    Ok(())
}
