//! Framing of control channel messages. Each message is JSON, prefixed with its
//! length as a 4-byte big-endian integer.

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::Message;

/// The largest frame either side accepts. Control messages are small.
pub const MAX_FRAME_LEN: u32 = 64 * 1024;

/// Writes one length-prefixed message.
pub async fn write_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    message: &Message,
) -> std::io::Result<()> {
    let json = serde_json::to_vec(message)?;
    let len = u32::try_from(json.len())
        .ok()
        .filter(|len| *len <= MAX_FRAME_LEN)
        .ok_or_else(|| std::io::Error::other("message too large"))?;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(&json).await?;
    writer.flush().await
}

/// Reads one length-prefixed message. Returns `None` when the peer finished
/// the stream cleanly before a new message started.
pub async fn read_message<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> std::io::Result<Option<Message>> {
    let mut len = [0u8; 4];
    match reader.read_exact(&mut len).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let len = u32::from_be_bytes(len);
    if len > MAX_FRAME_LEN {
        return Err(std::io::Error::other(format!(
            "frame of {len} bytes exceeds the limit"
        )));
    }
    let mut body = vec![0u8; len as usize];
    reader.read_exact(&mut body).await?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(std::io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_message_survives_a_round_trip() {
        let mut buffer = Vec::new();
        write_message(&mut buffer, &Message::Ping).await.unwrap();
        assert_eq!(
            &buffer[..4],
            &(br#"{"type":"ping"}"#.len() as u32).to_be_bytes()
        );

        let mut reader = buffer.as_slice();
        assert_eq!(
            read_message(&mut reader).await.unwrap(),
            Some(Message::Ping)
        );
        assert_eq!(read_message(&mut reader).await.unwrap(), None);
    }

    #[tokio::test]
    async fn an_unknown_type_is_read_as_unknown() {
        let body = br#"{"type":"from_the_future","x":1}"#;
        let mut frame = (body.len() as u32).to_be_bytes().to_vec();
        frame.extend_from_slice(body);

        let mut reader = frame.as_slice();
        assert_eq!(
            read_message(&mut reader).await.unwrap(),
            Some(Message::Unknown)
        );
    }

    #[tokio::test]
    async fn an_oversized_frame_is_refused() {
        let frame = (MAX_FRAME_LEN + 1).to_be_bytes();
        let mut reader = frame.as_slice();
        assert!(read_message(&mut reader).await.is_err());
    }
}
