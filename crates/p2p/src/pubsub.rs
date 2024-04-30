use libp2p::futures::io::{AsyncRead, AsyncReadExt, Error};
#[derive(Debug, Clone)]
pub enum PubsubMessage {
    Sync(String), // todo 改成Message(String) 解耦
}

impl PubsubMessage {
    pub async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> Result<Self, Error> {
        let mut buf = [0u8; 1];
        stream.read_exact(&mut buf).await?;
        let discriminator = buf[0];
        match discriminator {
            0 => {
                // let mut bytes = vec![0];
                let mut len_buf = [0u8; 4];
                stream.read_exact(&mut len_buf).await?;
                let len = u32::from_le_bytes(len_buf) as usize;
                let mut data = vec![0u8; len];
                stream.read_exact(&mut data).await?;
                let data = String::from_utf8(data).unwrap();
                Ok(Self::Sync(data))
            }
            _d => {
                todo!("error")
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Sync(data) => {
                let mut bytes = vec![0];
                let data = data.as_bytes();
                let len = data.len() as u32;
                bytes.extend_from_slice(&len.to_le_bytes());
                bytes.extend_from_slice(data);
                bytes
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let discriminator = bytes[0];
        match discriminator {
            0 => {
                let len = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
                let data = String::from_utf8(bytes[5..5 + len].to_vec()).unwrap();
                Ok(Self::Sync(data))
            }
            _d => {
                todo!("error")
            }
        }
    }
}
