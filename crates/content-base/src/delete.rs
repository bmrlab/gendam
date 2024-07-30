use crate::ContentBase;

pub struct DeletePayload {
    file_identifier: String,
}

impl DeletePayload {
    pub fn new(file_identifier: &str) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
        }
    }
}

impl ContentBase {
    pub async fn delete(&self, payload: DeletePayload) -> anyhow::Result<()> {
        todo!()
    }
}
