use crate::{metadata::ContentType, ContentBase};

pub struct UpsertPayload {
    file_path: String,
    file_identifier: String,
    content_type: ContentType,
}

impl ContentBase {
    pub async fn upsert(&self, payload: UpsertPayload) -> anyhow::Result<()> {
        todo!()
    }
}
