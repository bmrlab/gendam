use crate::ContentBase;

pub struct SearchPayload {
    query: String,
}

impl ContentBase {
    pub async fn search(&self, payload: SearchPayload) -> anyhow::Result<()> {
        todo!()
    }
}
