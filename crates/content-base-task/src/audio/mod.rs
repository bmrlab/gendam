pub mod trans_chunk;
pub mod trans_chunk_sum;
pub mod trans_chunk_sum_embed;
pub mod transcript;
pub mod thumbnail;
pub mod waveform;

use content_base_derive::ContentTask;
use storage_macro::Storage;
use strum_macros::{EnumIter, EnumString};
use thumbnail::AudioThumbnailTask;
use trans_chunk::AudioTransChunkTask;
use trans_chunk_sum::AudioTransChunkSumTask;
use trans_chunk_sum_embed::AudioTransChunkSumEmbedTask;
use transcript::AudioTranscriptTask;
use waveform::AudioWaveformTask;
use crate::task::ContentTaskType;

#[derive(Clone, Debug, EnumIter, EnumString, strum_macros::Display, ContentTask, Storage)]
#[strum(serialize_all = "kebab-case")]
pub enum AudioTaskType {
    Thumbnail(AudioThumbnailTask),
    Waveform(AudioWaveformTask),
    Transcript(AudioTranscriptTask),
    TransChunk(AudioTransChunkTask),
    TransChunkSum(AudioTransChunkSumTask),
    TransChunkSumEmbed(AudioTransChunkSumEmbedTask),
}

impl Into<ContentTaskType> for AudioTaskType {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Audio(self)
    }
}
