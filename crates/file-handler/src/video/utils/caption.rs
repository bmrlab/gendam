use super::save_text_embedding;
use crate::{
    search::payload::SearchPayload,
    video::{utils::get_frame_timestamp_from_path, CAPTION_FILE_EXTENSION, FRAME_FILE_EXTENSION},
};
use ai::{AsImageCaptionModel, AsTextEmbeddingModel};
use anyhow::Ok;
use prisma_lib::{video_frame, video_frame_caption, PrismaClient};
use qdrant_client::client::QdrantClient;
use serde_json::json;
use std::{io::Write, sync::Arc};
use strum_macros::AsRefStr;
use tracing::{debug, error};

#[derive(AsRefStr, Clone, Debug)]
pub(crate) enum CaptionMethod {
    BLIP,
    #[allow(dead_code)]
    YOLO,
    #[allow(dead_code)]
    Moondream,
}

pub(crate) async fn save_frames_caption(
    file_identifier: String,
    frames_dir: impl AsRef<std::path::Path>,
    image_caption: &dyn AsImageCaptionModel,
    client: Arc<PrismaClient>,
) -> anyhow::Result<()> {
    let frame_paths = std::fs::read_dir(frames_dir.as_ref())?
        .filter_map(|res| res.map(|e| e.path()).ok())
        .filter(|v| v.extension() == Some(std::ffi::OsStr::new(FRAME_FILE_EXTENSION)))
        .collect::<Vec<_>>();

    for path in frame_paths {
        debug!("get_frames_caption: {:?}", path);
        let frame_timestamp = get_frame_timestamp_from_path(&path)?;
        let client = client.clone();
        let file_identifier = file_identifier.clone();

        // check if caption exists, if it does, just skip it
        if let std::result::Result::Ok(Some(data)) = client
            .video_frame()
            .find_unique(
                video_frame::UniqueWhereParam::FileIdentifierTimestampEquals(
                    file_identifier.clone(),
                    frame_timestamp as i32,
                ),
            )
            .with(video_frame::video_clip::fetch())
            .exec()
            .await
        {
            if data.caption.is_some() {
                continue;
            }
        }

        match save_single_frame_caption(image_caption, path).await {
            anyhow::Result::Ok(caption) => {
                match client
                    .video_frame()
                    .upsert(
                        video_frame::UniqueWhereParam::FileIdentifierTimestampEquals(
                            file_identifier.clone(),
                            frame_timestamp as i32,
                        ),
                        (file_identifier.clone(), frame_timestamp as i32, vec![]),
                        vec![],
                    )
                    .exec()
                    .await
                {
                    anyhow::Result::Ok(video_frame) => {
                        if let Err(e) = client
                            .video_frame_caption()
                            .upsert(
                                video_frame_caption::UniqueWhereParam::VideoFrameIdMethodEquals(
                                    video_frame.id,
                                    CaptionMethod::BLIP.as_ref().into(),
                                ),
                                (
                                    caption.clone(),
                                    CaptionMethod::BLIP.as_ref().into(),
                                    video_frame::UniqueWhereParam::IdEquals(video_frame.id),
                                    vec![],
                                ),
                                vec![],
                            )
                            .exec()
                            .await
                        {
                            error!("failed to upsert video frame caption: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("failed to upsert video frame: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("failed to get frame caption: {:?}", e);
            }
        }
    }

    Ok(())
}

async fn save_single_frame_caption(
    image_caption: &dyn AsImageCaptionModel,
    path: impl AsRef<std::path::Path>,
) -> anyhow::Result<String> {
    let caption_path = path.as_ref().with_extension(CAPTION_FILE_EXTENSION);
    let caption = image_caption
        .get_images_caption_tx()
        .process_single(path.as_ref().to_owned())
        .await?;

    debug!("caption: {:?}", caption);

    // write into file
    let mut file = std::fs::File::create(caption_path)?;
    // here write as a json file, so that we can easily check the if file result is valid
    file.write_all(
        json!({
            "caption": caption
        })
        .to_string()
        .as_bytes(),
    )?;

    Ok(caption)
}

// pub(crate) async fn save_frames_tags(
//     file_identifier: String,
//     frames_dir: impl AsRef<std::path::Path>,
//     yolo_model: BatchHandler<YOLO>,
//     client: Arc<PrismaClient>,
// ) -> anyhow::Result<()> {
//     let frame_paths = std::fs::read_dir(frames_dir.as_ref())?
//         .map(|res| res.map(|e| e.path()))
//         .collect::<Result<Vec<_>, std::io::Error>>()?;

//     for path in frame_paths {
//         if path.extension() == Some(std::ffi::OsStr::new(FRAME_FILE_EXTENSION)) {
//             debug!("get_frames_tags: {:?}", path);
//             let yolo_model = yolo_model.clone();
//             let frame_timestamp = get_frame_timestamp_from_path(&path)?;
//             let client = client.clone();
//             let file_identifier = file_identifier.clone();

//             match save_single_frame_tags(yolo_model, path).await {
//                 anyhow::Result::Ok(caption) => {
//                     match client
//                         .video_frame()
//                         .upsert(
//                             video_frame::UniqueWhereParam::FileIdentifierTimestampEquals(
//                                 file_identifier.clone(),
//                                 frame_timestamp as i32,
//                             ),
//                             (file_identifier.clone(), frame_timestamp as i32, vec![]),
//                             vec![],
//                         )
//                         .exec()
//                         .await
//                     {
//                         anyhow::Result::Ok(video_frame) => {
//                             if let Err(e) = client
//                                 .video_frame_caption()
//                                 .upsert(
//                                     video_frame_caption::UniqueWhereParam::VideoFrameIdMethodEquals(
//                                         video_frame.id,
//                                         CaptionMethod::YOLO.as_ref().into(),
//                                     ),
//                                     (
//                                         caption.clone(),
//                                         CaptionMethod::YOLO.as_ref().into(),
//                                         video_frame::UniqueWhereParam::IdEquals(video_frame.id),
//                                         vec![],
//                                     ),
//                                     vec![],
//                                 )
//                                 .exec()
//                                 .await
//                             {
//                                 error!("failed to upsert video frame caption: {}", e);
//                             }
//                         }
//                         Err(e) => {
//                             error!("failed to upsert video frame: {}", e);
//                         }
//                     }
//                 }
//                 Err(e) => {
//                     error!("failed to get frame caption: {:?}", e);
//                 }
//             }
//         }
//     }

//     Ok(())
// }

// async fn save_single_frame_tags(
//     yolo_model: BatchHandler<YOLO>,
//     path: impl AsRef<std::path::Path>,
// ) -> anyhow::Result<String> {
//     let results = yolo_model.process_single(path.as_ref().to_owned()).await?;

//     let mut tags = results
//         .iter()
//         .map(|result| (result.get_class_name(), result.get_confidence()))
//         .collect::<Vec<_>>();
//     tags.sort_by(|a, b| a.1.total_cmp(&b.1));

//     let tags_caption = tags
//         .iter()
//         .map(|(tag, _)| tag.to_string())
//         .collect::<Vec<_>>()
//         .join(", ");

//     Ok(tags_caption)
// }

pub(crate) async fn save_frame_caption_embedding(
    file_identifier: String,
    client: Arc<PrismaClient>,
    frames_dir: impl AsRef<std::path::Path>,
    method: CaptionMethod,
    text_embedding: &dyn AsTextEmbeddingModel,
    qdrant: Arc<QdrantClient>,
) -> anyhow::Result<()> {
    let frame_paths = std::fs::read_dir(&frames_dir)?
        .filter_map(|res| res.map(|e| e.path()).ok())
        .filter(|v| v.extension() == Some(std::ffi::OsStr::new(FRAME_FILE_EXTENSION)))
        .collect::<Vec<_>>();

    for path in frame_paths {
        debug!(
            "save_frame_caption_embedding: {:?}, {}",
            path,
            method.as_ref()
        );
        let file_identifier = file_identifier.clone();
        let client = client.clone();
        let qdrant = qdrant.clone();
        let method = method.clone();

        let frame_timestamp = get_frame_timestamp_from_path(path)?;

        if let Err(e) = get_single_frame_caption_embedding(
            file_identifier,
            client,
            frame_timestamp,
            method,
            text_embedding,
            qdrant,
        )
        .await
        {
            error!("failed to save frame caption embedding: {:?}", e);
        }
    }

    Ok(())
}

/// Get caption embedding for a single frame
/// - The caption is read from prisma
/// - If the caption is empty, the embedding will be skipped
async fn get_single_frame_caption_embedding(
    file_identifier: String,
    client: Arc<PrismaClient>,
    timestamp: i64,
    method: CaptionMethod,
    text_embedding: &dyn AsTextEmbeddingModel,
    qdrant: Arc<QdrantClient>,
) -> anyhow::Result<()> {
    let x = {
        client
            .video_frame()
            .find_unique(
                video_frame::UniqueWhereParam::FileIdentifierTimestampEquals(
                    file_identifier.clone(),
                    timestamp as i32,
                ),
            )
            .with(video_frame::caption::fetch(vec![
                video_frame_caption::WhereParam::Method(
                    prisma_lib::read_filters::StringFilter::Equals(method.as_ref().into()),
                ),
            ]))
            .exec()
            .await
    };

    match x {
        std::result::Result::Ok(Some(res)) => {
            let caption = res
                .caption()?
                .first()
                .ok_or(anyhow::anyhow!("no caption record found"))?;

            if caption.caption.is_empty() {
                return Ok(());
            }

            let payload = SearchPayload::FrameCaption {
                id: caption.id as u64,
                file_identifier: file_identifier.clone(),
                timestamp,
                method: method.as_ref().into(),
            };
            save_text_embedding(
                &caption.caption,
                payload,
                text_embedding,
                qdrant,
                vector_db::DEFAULT_LANGUAGE_COLLECTION_NAME,
            )
            .await?;
        }
        std::result::Result::Ok(None) => {
            error!("failed to find frame caption");
        }
        Err(e) => {
            error!("failed to save frame caption embedding: {:?}", e);
        }
    }

    Ok(())
}
