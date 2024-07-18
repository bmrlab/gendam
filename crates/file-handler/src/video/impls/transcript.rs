use crate::{
    search::payload::SearchPayload,
    video::{VideoHandler, CHUNK_SUMMARIZATION_FILE_EXTENSION, EMBEDDING_FILE_EXTENSION},
};
use ai::{
    llm::{LLMInferenceParams, LLMMessage},
    Transcription,
};
use anyhow::bail;
use qdrant_client::qdrant::{PointStruct, UpsertPointsBuilder};
use serde_json::{json, Value};
use std::path::PathBuf;
use storage::Storage;
use tracing::{debug, error};

impl VideoHandler {
    pub fn get_transcript_chunks_path(&self) -> anyhow::Result<PathBuf> {
        let output_path = self
            .get_output_info_in_settings(&crate::video::VideoTaskType::TranscriptChunk)?
            .dir;
        let path = self.artifacts_dir.join(output_path).join("output.json");

        Ok(path)
    }

    pub fn get_transcript_chunk_summarization_path(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<PathBuf> {
        let output_path = self
            .get_output_info_in_settings(
                &crate::video::VideoTaskType::TranscriptChunkSummarization,
            )?
            .dir;
        let parent = self.artifacts_dir.join(output_path);

        Ok(parent.join(format!(
            "{}-{}.{}",
            start_timestamp, end_timestamp, CHUNK_SUMMARIZATION_FILE_EXTENSION
        )))
    }

    pub fn get_transcript_chunk_embedding_path(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<PathBuf> {
        let output_path = self
            .get_output_info_in_settings(&crate::video::VideoTaskType::TranscriptChunkEmbedding)?
            .dir;
        let parent = self.artifacts_dir.join(output_path);

        Ok(parent.join(format!(
            "{}-{}.{}",
            start_timestamp, end_timestamp, EMBEDDING_FILE_EXTENSION
        )))
    }

    pub fn get_transcript_chunk_summarization_embedding_path(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<PathBuf> {
        let output_path = self
            .get_output_info_in_settings(
                &crate::video::VideoTaskType::TranscriptChunkSummarizationEmbedding,
            )?
            .dir;
        let parent = self.artifacts_dir.join(output_path);

        Ok(parent.join(format!(
            "{}-{}.{}",
            start_timestamp, end_timestamp, EMBEDDING_FILE_EXTENSION
        )))
    }

    pub fn get_transcript_chunks(&self) -> anyhow::Result<Vec<Transcription>> {
        let transcript_chunks_path = self.get_transcript_chunks_path()?;
        let data = self.read_to_string(transcript_chunks_path)?;
        let data = serde_json::from_str(&data)?;
        Ok(data)
    }

    #[deprecated(note = "There should not be situation to call this function.")]
    pub fn get_transcript_chunk(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<String> {
        let transcript_chunks = self.get_transcript_chunks()?;

        for chunk in transcript_chunks {
            if chunk.start_timestamp == start_timestamp && chunk.end_timestamp == end_timestamp {
                return Ok(chunk.text);
            }
        }

        bail!("Failed to find transcript chunk")
    }

    pub fn get_transcript_chunk_embedding(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<Vec<f32>> {
        let embedding_path =
            self.get_transcript_chunk_embedding_path(start_timestamp, end_timestamp)?;
        self.get_embedding_from_file(embedding_path)
    }

    pub fn get_transcript_chunk_summarization(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<String> {
        let transcript_chunk_summarization_path =
            self.get_transcript_chunk_summarization_path(start_timestamp, end_timestamp)?;
        let content_str = self.read_to_string(transcript_chunk_summarization_path)?;
        let json_string: Value = serde_json::from_str(&content_str)?;
        let summarization = json_string["summarization"]
            .as_str()
            .ok_or(anyhow::anyhow!(
                "no summarization found in transcript chunk summarization file"
            ))?;
        Ok(summarization.to_string())
    }

    pub fn get_transcript_chunk_summarization_embedding(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<Vec<f32>> {
        let embedding_path =
            self.get_transcript_chunk_summarization_embedding_path(start_timestamp, end_timestamp)?;
        self.get_embedding_from_file(embedding_path)
    }

    pub(crate) async fn save_transcript_chunks(&self) -> anyhow::Result<()> {
        let tokenizer = self.tokenizer()?;
        let transcript = self.get_transcript()?;

        let mut chunks: Vec<Transcription> = vec![];
        let mut buffer: Vec<(Transcription, usize)> = vec![];
        let mut buffer_token_count = 0;
        let chunk_size = 100;

        let buffer_to_chunk = |buffer: &Vec<(Transcription, usize)>| Transcription {
            start_timestamp: buffer.first().expect("buffer not empty").0.start_timestamp,
            end_timestamp: buffer.last().expect("buffer not empty").0.end_timestamp,
            text: buffer
                .iter()
                .map(|v| v.0.text.clone())
                .collect::<Vec<_>>()
                .join("\n"),
        };

        for item in transcript.transcriptions.iter() {
            let current_token_count = tokenizer
                .encode(&*item.text, false)
                .map_err(|e| anyhow::anyhow!("failed to tokenize: {}", e))?
                .len();

            if current_token_count + buffer_token_count > chunk_size {
                // save current buffer to chunks
                let chunk = buffer_to_chunk(&buffer);
                chunks.push(chunk);

                // and reduce buffer to half of the chunk_size
                while buffer_token_count > chunk_size / 2 {
                    let first_item = buffer.remove(0);
                    buffer_token_count -= first_item.1;
                }
            }

            // push current item to buffer
            buffer.push((item.clone(), current_token_count));
            buffer_token_count += current_token_count;
        }

        // push remaining content to chunks
        if buffer.len() > 0 {
            let chunk = buffer_to_chunk(&buffer);
            chunks.push(chunk);
        }

        let output_path = self.get_transcript_chunks_path()?;
        self.write(output_path, serde_json::to_string(&chunks)?.into())
            .await?;

        Ok(())
    }

    pub(crate) async fn save_transcript_chunks_embedding(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub(crate) async fn save_transcript_chunks_summarization(&self) -> anyhow::Result<()> {
        let llm = self.llm()?.0;
        let chunks = self.get_transcript_chunks()?;
        let transcript = self.get_transcript()?;

        for i in 0..chunks.len() {
            let chunk = &chunks[i];
            debug!("save_transcript_chunks_summarization: {:?}", chunk);

            let previous_content = {
                if i == 0 {
                    "None".to_string()
                } else {
                    chunks[i - 1].text.clone()
                }
            };
            let user_prompt = format!(
                "Previous content:\n{}\n\nCurrent transcript:\n{}",
                previous_content, chunk.text
            );

            let mut response = llm
                .process_single((
                    vec![
                        LLMMessage::System(format!(r#"You are an assistant skilled in video transcript summarization.
You should try to summarize user input's transcript into a very short sentence.

Guidelines:
- Focus on essential information: Prioritize the transcript's core messages.
- Maintain clarity and conciseness: Craft your summary using accessible language.
- Capture the essence of the transcript.

Input:
- (optional) what is talking about in the previous video
- a piece of video transcript that need to be summarized

Input Example:
```
Previous content:
xxx

Current transcript:
xxx
```

Additional Rules:
- Content: just response with the short sentence only, do not start with hint or prompt, do not contain anything else, e.g., "The speaker is talking about his childhood."
- Focus: do not summarize the content in the previous video, focus on current piece of video transcript
- Word count: aim for a summarization with no more than 30 words.
- Language: summarization should be in the same language with input, which is {language}"#, language = transcript.language.as_ref())),
                        LLMMessage::User(user_prompt),
                    ],
                    LLMInferenceParams::default(),
                ))
                .await?;

            let summarization = response.to_string().await?;

            // write into file
            let output_path = self.get_transcript_chunk_summarization_path(
                chunk.start_timestamp,
                chunk.end_timestamp,
            )?;
            self.write(
                output_path,
                json!({
                    "summarization": summarization
                })
                .to_string()
                .into(),
            )
            .await?;
        }

        Ok(())
    }

    pub(crate) async fn save_transcript_chunks_summarization_embedding(
        &self,
    ) -> anyhow::Result<()> {
        let chunks = self.get_transcript_chunks()?;

        for chunk in chunks {
            if self
                .get_transcript_chunk_summarization_embedding(
                    chunk.start_timestamp,
                    chunk.end_timestamp,
                )
                .is_err()
            {
                let summarization = self.get_transcript_chunk_summarization(
                    chunk.start_timestamp,
                    chunk.end_timestamp,
                )?;
                let output_path = self.get_transcript_chunk_summarization_embedding_path(
                    chunk.start_timestamp,
                    chunk.end_timestamp,
                )?;
                if let Err(e) = self.save_text_embedding(&summarization, output_path).await {
                    error!(
                        "failed to save transcript chunk summarization embedding: {:?}",
                        e
                    );
                }
            }

            self.save_db_transcript_chunk_summarization_embedding(
                chunk.start_timestamp,
                chunk.end_timestamp,
            )
            .await?;
        }

        Ok(())
    }

    pub(crate) async fn save_db_transcript_chunk_summarization_embedding(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<()> {
        let qdrant = self.qdrant_client()?;
        let collection_name = self.language_collection_name()?;

        let embedding =
            self.get_transcript_chunk_summarization_embedding(start_timestamp, end_timestamp)?;

        let payload = SearchPayload::TranscriptChunkSummarization {
            file_identifier: self.file_identifier().to_string(),
            start_timestamp,
            end_timestamp,
        };

        let point = PointStruct::new(payload.get_uuid().to_string(), embedding, payload);
        qdrant
            .upsert_points(UpsertPointsBuilder::new(collection_name, vec![point]))
            .await?;

        Ok(())
    }
}
