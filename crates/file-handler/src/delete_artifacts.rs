use content_library::Library;
use prisma_client_rust::QueryError;
use prisma_lib::{video_clip, video_frame, video_transcript};
use qdrant_client::qdrant::{
    points_selector::PointsSelectorOneOf, Condition, Filter, PointsSelector,
};

use crate::video::{AUDIO_FILE_NAME, FRAME_DIR, TRANSCRIPT_FILE_NAME};

pub async fn handle_delete_artifacts(
    library: &Library,
    file_hashes: Vec<String>,
    vision_collection_name: &str,
    language_collection_name: &str,
    delete_asset: bool,
) -> anyhow::Result<()> {
    let file_hashes_clone = file_hashes.clone();

    // delete in prisma
    library
        .prisma_client()
        ._transaction()
        .run(|client| async move {
            client
                .video_frame()
                .delete_many(vec![video_frame::file_identifier::in_vec(
                    file_hashes.iter().map(|v| v.to_string()).collect(),
                )])
                .exec()
                .await?;

            client
                .video_transcript()
                .delete_many(vec![video_transcript::file_identifier::in_vec(
                    file_hashes.iter().map(|v| v.to_string()).collect(),
                )])
                .exec()
                .await?;

            client
                .video_clip()
                .delete_many(vec![video_clip::file_identifier::in_vec(
                    file_hashes.iter().map(|v| v.to_string()).collect(),
                )])
                .exec()
                .await?;

            std::result::Result::Ok(())
        })
        .await
        .map_err(|e: QueryError| e)?;

    // delete in qdrant
    let qdrant = library.qdrant_client();
    for file_hash in file_hashes_clone.iter() {
        qdrant
            .delete_points(
                language_collection_name,
                None,
                &PointsSelector {
                    points_selector_one_of: Some(PointsSelectorOneOf::Filter(Filter::all(vec![
                        Condition::matches("file_identifier", file_hash.to_string()),
                    ]))),
                },
                None,
            )
            .await?;
        qdrant
            .delete_points(
                vision_collection_name,
                None,
                &PointsSelector {
                    points_selector_one_of: Some(PointsSelectorOneOf::Filter(Filter::all(vec![
                        Condition::matches("file_identifier", file_hash.to_string()),
                    ]))),
                },
                None,
            )
            .await?;
    }

    // delete artifacts on file system
    for file_hash in file_hashes_clone.iter() {
        if delete_asset {
            // 直接删掉所有东西
            let path = library.artifacts_dir(&file_hash);
            std::fs::remove_dir_all(path).map_err(|e| {
                tracing::error!("failed to delete artifacts: {}", e);
                e
            })?;
        } else {
            // 仅删除生成结果
            let path = library.artifacts_dir(&file_hash);
            if let Err(e) = std::fs::remove_dir_all(path.join(FRAME_DIR)) {
                tracing::error!("failed to delete artifacts: {}", e);
            }
            if let Err(e) = std::fs::remove_file(path.join(TRANSCRIPT_FILE_NAME)) {
                tracing::error!("failed to delete artifacts: {}", e);
            };
            if let Err(e) = std::fs::remove_file(path.join(AUDIO_FILE_NAME)) {
                tracing::error!("failed to delete artifacts: {}", e);
            }
        }
    }

    Ok(())
}
