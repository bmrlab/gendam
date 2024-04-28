use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::whisper::TranscriptionLanguage;

use super::{AIModelLoader, AIModelTx};

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

pub trait AsAudioTranscriptModel: Send + Sync {
    fn get_audio_transcript_tx(&self) -> AIModelTx<AudioTranscriptInput, AudioTranscriptOutput>;
}

impl AsAudioTranscriptModel for AIModelLoader<AudioTranscriptInput, AudioTranscriptOutput> {
    fn get_audio_transcript_tx(&self) -> AIModelTx<AudioTranscriptInput, AudioTranscriptOutput> {
        self.tx.clone()
    }
}
