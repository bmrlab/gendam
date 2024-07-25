use crate::ContentBase;
use content_base_context::ContentBaseCtx;
use content_base_pool::TaskPool;

impl ContentBase {
    /// Create a new ContentBase with Context. The context will be cloned,
    /// so if need to modify context, a new ContentBase should be created.
    pub fn new(ctx: &ContentBaseCtx) -> anyhow::Result<Self> {
        let task_pool = TaskPool::new(ctx)?;

        Ok(Self {
            ctx: ctx.clone(),
            task_pool,
        })
    }
}
