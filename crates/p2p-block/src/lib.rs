#![allow(unused)] // TODO: This module is still in heavy development!

use std::{
    io,
    marker::PhantomData,
    path::{Path, PathBuf},
    string::FromUtf8Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use thiserror::Error;
use tokio::{
    fs::File,
    io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader},
};

use futures::{AsyncWrite, AsyncWriteExt};
use tracing::debug;

mod block;
mod block_size;
pub mod message;
pub mod proto;
mod sb_request;

pub use block::*;
pub use block_size::*;
pub use proto::{decode, encode};
pub use sb_request::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Msg<'a> {
    Block(Block<'a>),
    Cancelled,
}

impl<'a> Msg<'a> {
    pub async fn from_stream<'b>(
        stream: &mut (impl futures::io::AsyncReadExt + Unpin),
        data_buf: &'b mut [u8],
    ) -> Result<Msg<'a>, io::Error> {
        let mut buf = [0u8; 1];
        let _ = stream.read_exact(&mut buf).await?;
        match buf[0] {
            0 => Ok(Msg::Block(Block::from_stream(stream, data_buf).await?)),
            1 => Ok(Msg::Cancelled),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Invalid 'Msg' discriminator!",
            )),
        }
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Msg::Block(block) => {
                let mut bytes = Vec::new();
                bytes.push(0);
                bytes.extend(block.to_bytes());
                bytes
            }
            Msg::Cancelled => vec![1],
        }
    }
}

/// TODO
pub struct Transfer<'a, F> {
    reqs: &'a SpaceblockRequests,
    on_progress: F,
    total_offset: u64,
    total_bytes: u64,
    // TODO: Remove `i` plz
    i: usize,
    cancelled: &'a AtomicBool,
}

impl<'a, F> Transfer<'a, F>
where
    F: Fn(u8) + 'a,
{
    // TODO: Handle `req.range` correctly in this code

    pub fn new(req: &'a SpaceblockRequests, on_progress: F, cancelled: &'a AtomicBool) -> Self {
        Self {
            reqs: req,
            on_progress,
            total_offset: 0,
            total_bytes: req.requests.iter().map(|req| req.size).sum(),
            i: 0,
            cancelled,
        }
    }

    // TODO: Should `new` take in the streams too cause this means we `Stream` `SpaceblockRequest` could get outta sync.
    pub async fn send(
        &mut self,
        stream: &mut (impl futures::io::AsyncRead + futures::AsyncReadExt + AsyncWrite + Unpin),
        mut file: (impl AsyncBufRead + Unpin),
    ) -> Result<(), io::Error> {
        tracing::debug!("Sending file with total size of {}", self.total_bytes);
        // We manually implement what is basically a `BufReader` so we have more control
        let mut buf = vec![0u8; self.reqs.block_size.size() as usize];
        let mut offset: u64 = 0;

        loop {
            if self.cancelled.load(Ordering::Relaxed) {
                stream.write_all(&Msg::Cancelled.to_bytes()).await?;
                stream.flush().await?;
                return Ok(());
            }

            let read = file.read(&mut buf[..]).await?;
            tracing::debug!("Read {} bytes from file", read);
            self.total_offset += read as u64;
            (self.on_progress)(
                ((self.total_offset as f64 / self.total_bytes as f64) * 100.0) as u8,
            ); // SAFETY: Percent must be between 0 and 100

            if read == 0 {
                #[allow(clippy::panic)] // TODO: Remove panic
                                        // The file may have been modified during sender on the sender and we don't account for that.
                                        // TODO: Error handling + send error to remote
                assert!(
                    (offset + read as u64) == self.reqs.requests[self.i].size,
                    "File sending has stopped but it doesn't match the expected length!"
                );

                return Ok(());
            }

            let block = Block {
                offset,
                size: read as u64,
                data: &buf[..read],
            };
            debug!(
                "Sending block at offset {} of size {}",
                block.offset, block.size
            );
            offset += read as u64;

            let msg = &Msg::Block(block).to_bytes();

            stream.write_all(msg).await?;
            stream.flush().await?;
            let mut buf = [0u8; 1];
            let _ = stream.read_exact(&mut buf).await?;
            match buf[0] {
                // Continue sending
                0 => {}
                // Cancelled by user
                1 => {
                    debug!("Receiver cancelled Spacedrop transfer!");
                    return Ok(());
                }
                // Transfer complete
                2 => return Ok(()),
                _ => todo!(),
            }
        }
    }

    // TODO: Timeout on receiving/sending
    pub async fn receive(
        &mut self,
        stream: &mut (impl futures::io::AsyncRead + futures::io::AsyncWrite + Unpin),
        mut file: (impl tokio::io::AsyncWrite + tokio::io::AsyncWriteExt + Unpin),
        // TODO: Proper error type
    ) -> Result<(), io::Error> {
        // We manually implement what is basically a `BufReader` so we have more control
        let mut data_buf = vec![0u8; self.reqs.block_size.size() as usize];
        let mut offset: u64 = 0;

        if self.reqs.requests[self.i].size == 0 {
            self.i += 1;
            return Ok(());
        }

        // TODO: Prevent loop being a DOS vector
        loop {
            if self.cancelled.load(Ordering::Relaxed) {
                stream.write(&[1u8]).await?;
                stream.flush().await?;
                return Ok(());
            }

            // TODO: Timeout if nothing is being received
            let msg = Msg::from_stream(stream, &mut data_buf).await?;

            tracing::info!("Transfer Received {:?}", msg);

            match msg {
                Msg::Block(block) => {
                    self.total_offset += block.size;
                    (self.on_progress)(
                        ((self.total_offset as f64 / self.total_bytes as f64) * 100.0) as u8,
                    ); // SAFETY: Percent must be between 0 and 100

                    debug!(
                        "Received block at offset {} of size {}",
                        block.offset, block.size
                    );
                    offset += block.size;

                    file.write_all(&data_buf[..block.size as usize]).await?;

                    let req = self.reqs.requests.get(self.i).ok_or_else(|| {
                        debug!("Vector read out of bounds!");
                        io::ErrorKind::Other
                    })?;
                    // TODO: Should this be `read == 0`
                    if offset == req.size {
                        break;
                    }

                    stream
                        .write(&[u8::from(self.cancelled.load(Ordering::Relaxed))])
                        .await?;
                    stream.flush().await?;
                }
                Msg::Cancelled => {
                    debug!("Sender cancelled Spacedrop transfer!");
                    return Ok(());
                }
            }
        }

        stream.write(&[2u8]).await?;
        stream.flush().await?;
        file.flush().await?;
        self.i += 1;

        Ok(())
    }
}