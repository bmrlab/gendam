use std::io;

use futures::io::{AsyncRead, AsyncReadExt};

/// TODO
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSize(u32); // Max block size is gonna be 3.9GB which is stupidly overkill

impl BlockSize {
	// TODO: Validating `BlockSize` are multiple of 2, i think. Idk why but BEP does it.

	pub async fn from_stream(stream: &mut (impl AsyncRead + Unpin)) -> io::Result<Self> {
		read_u32_le(stream).await.map(Self)
	}

	#[must_use]
	pub fn to_bytes(&self) -> [u8; 4] {
		self.0.to_le_bytes()
	}

	#[must_use]
	pub fn from_size(size: u64) -> Self {
		// TODO: Something like: https://docs.syncthing.net/specs/bep-v1.html#selection-of-block-size
		Self(131_072) // 128 KiB
	}

	/// This is super dangerous as it doesn't enforce any assumptions of the protocol and is designed just for tests.
	#[cfg(test)]
	#[must_use]
	pub fn dangerously_new(size: u32) -> Self {
		Self(size)
	}

	#[must_use]
	pub fn size(&self) -> u32 {
		self.0
	}
}


async fn read_u32_le(stream: &mut (impl AsyncRead + Unpin)) -> Result<u32, io::Error> {
	let mut buf = [0; 4];  
	stream.read_exact(&mut buf).await?;
	Ok(u32::from_le_bytes(buf))
  }