use futures::AsyncRead;
use p2p_block::StreamData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareInfo {
    pub file_count: usize,
}

impl StreamData for ShareInfo {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let file_count = usize::from_stream(stream).await?;
        Ok(Self { file_count })
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.file_count.to_le_bytes().to_vec()
    }
}
