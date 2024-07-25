use super::DocTaskType;
use crate::{ContentTask, ContentTaskType};
use async_trait::async_trait;
use storage_macro::Storage;

#[derive(Storage, Clone, Debug)]
pub struct DocChunkTask {}

impl Default for DocChunkTask {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait]
impl ContentTask for DocChunkTask {
    fn artifacts_output_path(
        &self,
        file_info: &crate::FileInfo,
        ctx: &crate::ContentBase,
    ) -> anyhow::Result<std::path::PathBuf> {
        todo!()
    }

    async fn inner_run(
        &self,
        file_info: &crate::FileInfo,
        ctx: &crate::ContentBase,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn model_name(&self, ctx: &crate::ContentBase) -> String {
        "".into()
    }
}

impl Into<ContentTaskType> for DocChunkTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Doc(DocTaskType::Chunk(self))
    }
}
