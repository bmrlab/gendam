use crate::{
    search::payload::SearchPayload,
    video::{decoder, VideoHandler, VideoTaskType, EMBEDDING_FILE_EXTENSION},
};
use ai::AudioTranscriptOutput;
use qdrant_client::qdrant::PointStruct;
use serde_json::json;
use std::path::PathBuf;
use storage::prelude::*;
use tracing::error;

impl VideoHandler {
    pub(crate) fn get_audio_path(&self) -> anyhow::Result<PathBuf> {
        let output_path = self.get_output_info_in_settings(&VideoTaskType::Audio)?.dir;
        Ok(self.artifacts_dir.join(output_path).join("audio.wav"))
    }

    pub fn get_transcript_path(&self) -> anyhow::Result<PathBuf> {
        let output_path = self
            .get_output_info_in_settings(&VideoTaskType::Transcript)?
            .dir;
        let path = self.artifacts_dir.join(output_path).join("output.json");

        Ok(path)
    }

    pub fn get_transcript(&self) -> anyhow::Result<AudioTranscriptOutput> {
        let transcript_path = self.get_transcript_path()?;

        let data = self.read_to_string(transcript_path)?;
        let transcription: AudioTranscriptOutput = serde_json::from_str(data.as_str())?;

        Ok(transcription)
    }

    pub(crate) fn get_transcript_embedding_path(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<PathBuf> {
        let output_path = self
            .get_output_info_in_settings(&VideoTaskType::TranscriptEmbedding)?
            .dir;
        let parent = self.artifacts_dir.join(output_path);

        Ok(parent.join(format!(
            "{}-{}.{}",
            start_timestamp, end_timestamp, EMBEDDING_FILE_EXTENSION
        )))
    }

    pub(crate) fn get_transcript_embedding(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<Vec<f32>> {
        let embedding_path = self.get_transcript_embedding_path(start_timestamp, end_timestamp)?;
        self.get_embedding_from_file(embedding_path)
    }

    /// Extract audio from video and save results
    pub(crate) async fn save_audio(&self) -> anyhow::Result<()> {
        if self.check_artifacts(&VideoTaskType::Audio) {
            return Ok(());
        }

        let audio_path = self.get_audio_path()?;

        #[cfg(feature = "ffmpeg-binary")]
        {
            let video_decoder = decoder::VideoDecoder::new(&self.video_path)?;
            video_decoder.save_video_audio(audio_path).await?;
        }

        #[cfg(feature = "ffmpeg-dylib")]
        {
            let video_decoder = decoder::VideoDecoder::new(&self.video_path);
            video_decoder.save_video_audio(audio_path).await?;
        }

        Ok(())
    }

    /// Convert audio of the video into text
    /// **This requires extracting audio in advance**
    ///
    /// This will also save results:
    /// - Save into disk (a folder named by `library` and `video_file_hash`)
    /// - Save into prisma `VideoTranscript` model
    pub(crate) async fn save_transcript(&self) -> anyhow::Result<()> {
        if self.get_transcript().is_ok() {
            return Ok(());
        }

        let audio_transcript = self.audio_transcript()?.0;
        let result = audio_transcript
            .process_single(self.get_audio_path()?)
            .await?;

        let result_path = self.get_transcript_path()?;

        self.write(result_path, serde_json::to_string(&result)?.into())
            .await?;

        Ok(())
    }

    pub(crate) async fn save_transcript_embedding(&self) -> anyhow::Result<()> {
        let transcript_results = self.get_transcript()?.transcriptions;

        for item in transcript_results {
            // if item is some like [MUSIC], just skip it
            // TODO need to make sure all filter rules
            if item.text.starts_with("[") || item.text.starts_with("(") {
                continue;
            }

            if self
                .get_transcript_embedding(item.start_timestamp, item.end_timestamp)
                .is_err()
            {
                if let Err(e) = self
                    .save_text_embedding(
                        &item.text,
                        self.get_transcript_embedding_path(
                            item.start_timestamp,
                            item.end_timestamp,
                        )?,
                    )
                    .await
                {
                    error!("failed to save transcript embedding: {:?}", e);
                }
            }

            self.save_db_single_transcript_embedding(item.start_timestamp, item.end_timestamp)
                .await?;
        }

        Ok(())
    }

    pub(crate) async fn save_db_single_transcript_embedding(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<()> {
        let qdrant = self.qdrant_client()?;
        let collection_name = self.language_collection_name()?;

        let embedding = self.get_transcript_embedding(start_timestamp, end_timestamp)?;

        let audio_transcript_model_name = self.audio_transcript()?.1;

        let payload = SearchPayload::Transcript {
            file_identifier: self.file_identifier().to_string(),
            start_timestamp,
            end_timestamp,
            method: audio_transcript_model_name.into(),
        };
        let point = PointStruct::new(
            payload.get_uuid().to_string(),
            embedding.clone(),
            json!(payload)
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid payload"))?,
        );
        qdrant
            .upsert_points(collection_name, None, vec![point], None)
            .await?;

        Ok(())
    }
}
