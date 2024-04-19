use std::{fs::File, io::BufReader};

use ai::Transcription;
use prisma_lib::video_transcript;
use tokio::io::AsyncWriteExt;
use tracing::error;

use crate::{
    search::payload::SearchPayload,
    video::{decoder, VideoHandler, AUDIO_FILE_NAME, TRANSCRIPT_FILE_NAME},
};

impl VideoHandler {
    /// Extract audio from video and save results
    /// - Save into disk (a folder named by `library` and `video_file_hash`)
    pub(crate) async fn save_audio(&self) -> anyhow::Result<()> {
        #[cfg(feature = "ffmpeg-binary")]
        {
            let video_decoder = decoder::VideoDecoder::new(&self.video_path)?;
            video_decoder
                .save_video_audio(self.artifacts_dir.join(AUDIO_FILE_NAME))
                .await?;
        }

        #[cfg(feature = "ffmpeg-dylib")]
        {
            let video_decoder = decoder::VideoDecoder::new(&self.video_path);
            video_decoder.save_video_audio(&self.audio_path).await?;
        }

        Ok(())
    }

    pub(crate) async fn delete_audio(&self) -> anyhow::Result<()> {
        let audio_path = self.artifacts_dir.join(AUDIO_FILE_NAME);

        if audio_path.exists() {
            std::fs::remove_file(audio_path)?;
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
        let result = self
            .audio_transcript()?
            .get_audio_transcript_tx()
            .process_single(self.artifacts_dir.join(AUDIO_FILE_NAME))
            .await?;

        // write results into json file
        let mut file =
            tokio::fs::File::create(self.artifacts_dir.join(TRANSCRIPT_FILE_NAME)).await?;
        let json = serde_json::to_string(&result.transcriptions)?;
        file.write_all(json.as_bytes()).await?;

        for item in result.transcriptions {
            let file_identifier = self.file_identifier();
            let client = self.library.prisma_client();

            let x = {
                client
                    .video_transcript()
                    .upsert(
                        video_transcript::file_identifier_start_timestamp_end_timestamp(
                            file_identifier.to_string(),
                            item.start_timestamp as i32,
                            item.end_timestamp as i32,
                        ),
                        (
                            file_identifier.to_string(),
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

    pub(crate) async fn delete_transcript(&self) -> anyhow::Result<()> {
        let client = self.library.prisma_client();
        let file_identifier = self.file_identifier();

        client
            .video_transcript()
            .delete_many(vec![video_transcript::file_identifier::equals(
                file_identifier.to_string(),
            )])
            .exec()
            .await?;

        tokio::fs::remove_file(self.artifacts_dir.join(TRANSCRIPT_FILE_NAME)).await?;

        Ok(())
    }

    pub(crate) async fn save_transcript_embedding(&self) -> anyhow::Result<()> {
        // utils::transcript::save_transcript_embedding(self).await?;
        let transcript_path = self.artifacts_dir.join(TRANSCRIPT_FILE_NAME);

        let transcript_results: Vec<Transcription> = {
            let file = File::open(transcript_path)?;
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

            // write data using prisma
            // here use write to make sure only one thread can using prisma client
            let x = {
                self.library
                    .prisma_client()
                    .video_transcript()
                    .upsert(
                        video_transcript::file_identifier_start_timestamp_end_timestamp(
                            self.file_identifier().to_string(),
                            item.start_timestamp as i32,
                            item.end_timestamp as i32,
                        ),
                        (
                            self.file_identifier().to_string(),
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
                        file_identifier: self.file_identifier().to_string(),
                        start_timestamp: item.start_timestamp,
                        end_timestamp: item.end_timestamp,
                    };
                    if let Err(e) = self
                        .save_text_embedding(
                            &item.text, // but embedding english
                            payload,
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

    pub(crate) async fn delete_transcript_embedding(&self) -> anyhow::Result<()> {
        self.delete_embedding(crate::SearchRecordType::Transcript)
            .await
    }
}
