use super::save_text_embedding;
use crate::index::EmbeddingIndex;
use ai::{clip::CLIP, whisper::WhisperItem};
use prisma_lib::{video_transcript, PrismaClient};
use std::{fs::File, io::BufReader, path::Path, sync::Arc};
use tokio::sync::RwLock;
use tracing::error;

pub async fn get_transcript_embedding(
    file_identifier: String,
    client: Arc<RwLock<PrismaClient>>,
    path: impl AsRef<Path>,
    clip_model: Arc<RwLock<CLIP>>,
    embedding_index: Arc<EmbeddingIndex>,
) -> anyhow::Result<()> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `WhisperItem`
    let whisper_results: Vec<WhisperItem> = serde_json::from_reader(reader)?;

    let clip_model = clip_model.clone();
    let embedding_index = embedding_index.clone();

    let mut join_set = tokio::task::JoinSet::new();

    for item in whisper_results {
        // if item is some like [MUSIC], just skip it
        // TODO need to make sure all filter rules
        if item.text.starts_with("[") || item.text.starts_with("(") {
            continue;
        }

        let clip_model = Arc::clone(&clip_model);
        let file_identifier = file_identifier.clone();
        let embedding_index = Arc::clone(&embedding_index);
        let client = client.clone();

        join_set.spawn(async move {
            // write data using prisma
            // here use write to make sure only one thread can using prisma client
            let client = client.write().await;
            let x = client.video_transcript().upsert(
                video_transcript::file_identifier_start_timestamp_end_timestamp(
                    file_identifier.clone(),
                    item.start_timestamp as i32,
                    item.end_timestamp as i32,
                ),
                (
                    file_identifier,
                    item.start_timestamp as i32,
                    item.end_timestamp as i32,
                    item.text.clone(),
                    vec![],
                ),
                vec![],
            );

            match x.exec().await {
                std::result::Result::Ok(res) => {
                    if let Err(e) =
                        save_text_embedding(&item.text, res.id as u64, clip_model, embedding_index)
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
