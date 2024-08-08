pub mod transform;
pub mod chunk;
pub mod chunk_sum;
pub mod chunk_sum_embed;

use crate::ContentTaskType;
use transform::WebPageTransformTask;
use chunk::WebPageChunkTask;
use chunk_sum::WebPageChunkSumTask;
use chunk_sum_embed::WebPageChunkSumEmbedTask;
use content_base_derive::ContentTask;
use storage_macro::Storage;
use strum::{EnumIter, EnumString};

#[derive(Clone, Debug, EnumIter, EnumString, strum_macros::Display, ContentTask, Storage)]
#[strum(serialize_all = "kebab-case")]
pub enum WebPageTaskType {
    Transform(WebPageTransformTask),
    Chunk(WebPageChunkTask),
    ChunkSum(WebPageChunkSumTask),
    ChunkSumEmbed(WebPageChunkSumEmbedTask),
}

impl Into<ContentTaskType> for WebPageTaskType {
    fn into(self) -> ContentTaskType {
        ContentTaskType::WebPage(self)
    }
}
