use std::sync::Arc;

use crate::ContentBase;
use content_base_context::ContentBaseCtx;
use content_base_pool::TaskPool;
use qdrant_client::Qdrant;

impl ContentBase {
    /// Create a new ContentBase with Context. The context will be cloned,
    /// so if need to modify context, a new ContentBase should be created.
    pub fn new(
        ctx: &ContentBaseCtx,
        qdrant_client: Arc<Qdrant>,
        language_collection_name: &str,
        vision_collection_name: &str,
    ) -> anyhow::Result<Self> {
        let task_pool = TaskPool::new(ctx, None)?;
        Ok(Self {
            ctx: ctx.clone(),
            task_pool,
            qdrant: qdrant_client,
            language_collection_name: language_collection_name.to_string(),
            vision_collection_name: vision_collection_name.to_string(),
        })
    }

    pub fn ctx(&self) -> &ContentBaseCtx {
        &self.ctx
    }
}
