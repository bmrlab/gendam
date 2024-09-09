pub mod chunk;
pub mod chunk_sum;
pub mod chunk_sum_embed;

use crate::ContentTaskType;
use chunk::RawTextChunkTask;
use chunk_sum::RawTextChunkSumTask;
use chunk_sum_embed::RawTextChunkSumEmbedTask;
use content_base_derive::ContentTask;
use storage_macro::Storage;
use strum::{EnumIter, EnumString};

#[derive(Clone, Debug, EnumIter, EnumString, strum_macros::Display, ContentTask, Storage)]
#[strum(serialize_all = "kebab-case")]
pub enum RawTextTaskType {
    Chunk(RawTextChunkTask),
    ChunkSum(RawTextChunkSumTask),
    ChunkSumEmbed(RawTextChunkSumEmbedTask),
}

impl Into<ContentTaskType> for RawTextTaskType {
    fn into(self) -> ContentTaskType {
        ContentTaskType::RawText(self)
    }
}
