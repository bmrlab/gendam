use super::save_text_embedding;
use crate::{
    search::payload::SearchPayload,
    video::{AUDIO_FILE_NAME, TRANSCRIPT_FILE_NAME},
};
use ai::{AsAudioTranscriptModel, AsTextEmbeddingModel, Transcription};
use prisma_lib::{video_transcript, PrismaClient};
use qdrant_client::client::QdrantClient;
use std::{fs::File, io::BufReader, path::Path, sync::Arc};
use tokio::io::AsyncWriteExt;
use tracing::error;

pub async fn save_transcript(
    artifacts_dir: impl AsRef<std::path::Path>,
    file_identifier: String,
    client: Arc<PrismaClient>,
    audio_transcript: &dyn AsAudioTranscriptModel,
) -> anyhow::Result<()> {
    let result = audio_transcript
        .get_audio_transcript_tx()
        .process_single(artifacts_dir.as_ref().join(AUDIO_FILE_NAME))
        .await?;

    // write results into json file
    let mut file =
        tokio::fs::File::create(artifacts_dir.as_ref().join(TRANSCRIPT_FILE_NAME)).await?;
    let json = serde_json::to_string(&result.transcriptions)?;
    file.write_all(json.as_bytes()).await?;

    for item in result.transcriptions {
        let file_identifier = file_identifier.clone();
        let client = client.clone();

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
    }

    Ok(())
}

pub async fn save_transcript_embedding(
    file_identifier: String,
    client: Arc<PrismaClient>,
    path: impl AsRef<Path>,
    text_embedding: &dyn AsTextEmbeddingModel,
    qdrant: Arc<QdrantClient>,
    collection_name: &str,
) -> anyhow::Result<()> {
    let transcript_results: Vec<Transcription> = {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        // Read the JSON contents of the file as an instance of `Transcription`
        serde_json::from_reader(reader)?
    };

    for item in transcript_results {
        // if item is some like [MUSIC], just skip it
        // TODO need to make sure all filter rules
        if item.text.starts_with("[") || item.text.starts_with("(") {
            continue;
        }

        let file_identifier = file_identifier.clone();
        let client = client.clone();
        let qdrant = qdrant.clone();

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
                    text_embedding,
                    qdrant,
                    collection_name,
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
    }

    Ok(())
}
