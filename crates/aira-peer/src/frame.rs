//! Length-prefixed JSON frames over TCP (u32 big-endian).

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::error::PeerError;

/// Maximum accepted frame payload (1 MiB).
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;

/// Write one length-prefixed frame.
pub async fn write_frame(stream: &mut TcpStream, payload: &[u8]) -> Result<(), PeerError> {
    if payload.len() > MAX_FRAME_BYTES {
        return Err(PeerError::FrameTooLarge(payload.len()));
    }
    let len = (payload.len() as u32).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(payload).await?;
    stream.flush().await?;
    Ok(())
}

/// Read one length-prefixed frame.
pub async fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, PeerError> {
    let mut len_buf = [0u8; 4];
    if let Err(e) = stream.read_exact(&mut len_buf).await {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            return Err(PeerError::TruncatedFrame);
        }
        return Err(e.into());
    }
    let len = u32::from_be_bytes(len_buf) as usize;
    if len == 0 {
        return Err(PeerError::TruncatedFrame);
    }
    if len > MAX_FRAME_BYTES {
        return Err(PeerError::FrameTooLarge(len));
    }
    let mut buf = vec![0u8; len];
    if let Err(e) = stream.read_exact(&mut buf).await {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            return Err(PeerError::TruncatedFrame);
        }
        return Err(e.into());
    }
    Ok(buf)
}

/// Serialize `value` as JSON and write as one frame.
pub async fn write_json<T: serde::Serialize>(
    stream: &mut TcpStream,
    value: &T,
) -> Result<(), PeerError> {
    let bytes = serde_json::to_vec(value)?;
    write_frame(stream, &bytes).await
}

/// Read one frame and deserialize JSON.
pub async fn read_json<T: serde::de::DeserializeOwned>(
    stream: &mut TcpStream,
) -> Result<T, PeerError> {
    let bytes = read_frame(stream).await?;
    Ok(serde_json::from_slice(&bytes)?)
}
