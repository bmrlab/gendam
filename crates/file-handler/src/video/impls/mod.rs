pub mod artifacts;
pub mod audio;
pub mod caption;
pub mod frame;

use storage::StorageTrait;

use super::VideoHandler;
use std::path::Path;

pub(self) fn get_frame_timestamp_from_path(
    path: impl AsRef<std::path::Path>,
) -> anyhow::Result<i64> {
    let file_name = path
        .as_ref()
        .file_name()
        .ok_or(anyhow::anyhow!("invalid path"))?
        .to_string_lossy()
        .to_string();

    let frame_timestamp: i64 = file_name
        .split(".")
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    Ok(frame_timestamp)
}

impl VideoHandler {
    pub(self) async fn save_text_embedding(
        &self,
        text: &str,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let embedding = self
            .text_embedding()?
            .0
            .get_texts_embedding_tx()
            .process_single(text.to_string())
            .await?;

        self.write(
            path.as_ref().to_path_buf(),
            serde_json::to_string(&embedding)?.into(),
        )
        .await?;
        Ok(())
    }

    pub fn get_embedding_from_file(&self, path: impl AsRef<Path>) -> anyhow::Result<Vec<f32>> {
        let embedding: String = self.read_to_string(path.as_ref().to_path_buf())?;
        serde_json::from_str::<Vec<f32>>(&embedding).map_err(|e| anyhow::anyhow!(e))
    }
}
