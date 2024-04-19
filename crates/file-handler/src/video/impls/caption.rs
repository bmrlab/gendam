use crate::{
    search::payload::SearchPayload,
    video::{
        impls::get_frame_timestamp_from_path, VideoHandler, CAPTION_FILE_EXTENSION, FRAME_DIR,
        FRAME_FILE_EXTENSION,
    },
};
use ai::AsImageCaptionModel;
use anyhow::bail;
use prisma_lib::{video_frame, video_frame_caption};
use serde_json::json;
use std::{io::Write, path::Path};
use strum_macros::AsRefStr;
use tracing::{debug, error};

#[derive(AsRefStr, Clone)]
pub enum CaptionMethod {
    BLIP,
    #[allow(dead_code)]
    Moondream,
}

impl VideoHandler {
    /// Save frames' captions of video
    /// **this requires extracting frames in advance**
    ///
    /// The captions will be saved:
    /// - To disk: as `.caption` file in the same place with frame file
    /// - To prisma `VideoFrameCaption` model
    pub(crate) async fn save_frames_caption(&self) -> anyhow::Result<()> {
        let file_identifier = self.file_identifier.clone();
        let frames_dir = self.artifacts_dir.join(FRAME_DIR);
        let image_caption = self.image_caption()?;
        let client = self.library.prisma_client();

        let frame_paths = std::fs::read_dir(frames_dir)?
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

            match self.save_single_frame_caption(path, image_caption).await {
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
        &self,
        path: impl AsRef<Path>,
        image_caption: &dyn AsImageCaptionModel,
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

    /// Save frame caption embedding into qdrant
    /// this requires extracting frames and get captions in advance
    pub(crate) async fn save_frame_caption_embedding(&self) -> anyhow::Result<()> {
        let frames_dir = self.artifacts_dir.join(FRAME_DIR);
        let method = CaptionMethod::BLIP;

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
            let method = method.clone();

            let frame_timestamp = get_frame_timestamp_from_path(path)?;

            if let Err(e) = self
                .save_single_frame_caption_embedding(frame_timestamp, method)
                .await
            {
                error!("failed to save frame caption embedding: {:?}", e);
            }
        }

        Ok(())
    }

    async fn save_single_frame_caption_embedding(
        &self,
        frame_timestamp: i64,
        method: CaptionMethod,
    ) -> anyhow::Result<()> {
        let client = self.library.prisma_client();
        let file_identifier = self.file_identifier.clone();

        let x = {
            client
                .video_frame()
                .find_unique(
                    video_frame::UniqueWhereParam::FileIdentifierTimestampEquals(
                        file_identifier.clone(),
                        frame_timestamp as i32,
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
                    timestamp: frame_timestamp,
                    method: method.as_ref().into(),
                };
                self.save_text_embedding(&caption.caption, payload).await?;
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

    pub(crate) async fn delete_frames_caption(&self) -> anyhow::Result<()> {
        let client = self.library.prisma_client();
        let file_identifier = self.file_identifier.clone();

        let frames = client
            .video_frame()
            .find_many(vec![video_frame::file_identifier::equals(
                file_identifier.to_string(),
            )])
            .exec()
            .await?;

        if let Err(e) = client
            .video_frame_caption()
            .delete_many(vec![video_frame_caption::video_frame_id::in_vec(
                frames.iter().map(|frame| frame.id).collect::<Vec<_>>(),
            )])
            .exec()
            .await
        {
            bail!("failed to delete video frame caption: {}", e);
        }

        Ok(())
    }

    pub(crate) async fn delete_frame_caption_embedding(&self) -> anyhow::Result<()> {
        self.delete_embedding(crate::SearchRecordType::FrameCaption)
            .await
    }
}
