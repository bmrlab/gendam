use super::AIModel;
use crate::whisper::TranscriptionLanguage;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transcription {
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub text: String,
}

pub type AudioTranscriptInput = PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct AudioTranscriptOutput {
    pub language: TranscriptionLanguage,
    pub transcriptions: Vec<Transcription>,
}

pub type AudioTranscriptModel = AIModel<AudioTranscriptInput, AudioTranscriptOutput>;
