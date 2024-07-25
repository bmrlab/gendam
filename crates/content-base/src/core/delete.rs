use crate::ContentBase;

pub struct DeletePayload {
    file_identifier: String,
}

impl ContentBase {
    pub async fn delete(&self, payload: DeletePayload) -> anyhow::Result<()> {
        todo!()
    }
}
