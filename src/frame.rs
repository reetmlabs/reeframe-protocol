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

    fn session_messages() -> Vec<Message> {
        vec![
            Message::SessionOffer {
                session_id: "0b6f7c3e-4c1a-4f0e-9a51-2d3c1f6b8e10".into(),
                sdp: "v=0\r\na=group:BUNDLE 0\r\n".into(),
                ice_servers: vec![
                    crate::IceServer {
                        urls: vec!["stun:stun.example.com:3478".into()],
                        username: None,
                        credential: None,
                    },
                    crate::IceServer {
                        urls: vec!["turn:turn.example.com:3478".into()],
                        username: Some("user".into()),
                        credential: Some("secret".into()),
                    },
                ],
                site_token: "header.claims.signature".into(),
            },
            Message::SessionAnswer {
                session_id: "s1".into(),
                sdp: "v=0\r\n".into(),
            },
            Message::SessionCandidate {
                session_id: "s1".into(),
                candidate: "candidate:1 1 udp 2122260223 192.0.2.1 54400 typ host".into(),
                sdp_mid: Some("0".into()),
                sdp_mline_index: Some(0),
            },
            Message::SessionCandidate {
                session_id: "s1".into(),
                candidate: "candidate:1".into(),
                sdp_mid: None,
                sdp_mline_index: None,
            },
            Message::SessionClose {
                session_id: "s1".into(),
                reason: "client left".into(),
            },
            Message::SessionError {
                session_id: "s1".into(),
                code: crate::session_error::INVALID_OFFER.into(),
                message: "no video section".into(),
            },
        ]
    }

    #[tokio::test]
    async fn session_messages_survive_the_quic_framing() {
        let mut buffer = Vec::new();
        for message in session_messages() {
            write_message(&mut buffer, &message).await.unwrap();
        }
        let mut reader = buffer.as_slice();
        for message in session_messages() {
            assert_eq!(read_message(&mut reader).await.unwrap(), Some(message));
        }
        assert_eq!(read_message(&mut reader).await.unwrap(), None);
    }

    #[test]
    fn session_messages_survive_the_websocket_encoding() {
        for message in session_messages() {
            assert_eq!(Message::from_json(&message.to_json()).unwrap(), message);
        }
    }

    // A real SDP is under 10 KiB. Twice that, with CRLF line ends that JSON
    // escapes, plus TURN servers and a site JWT, must still fit in one frame.
    #[tokio::test]
    async fn a_large_session_offer_fits_in_one_frame() {
        let sdp = "a=candidate:1 1 udp 2122260223 192.0.2.1 54400 typ host\r\n".repeat(400);
        assert!(sdp.len() > 20 * 1024);
        let offer = Message::SessionOffer {
            session_id: "0b6f7c3e-4c1a-4f0e-9a51-2d3c1f6b8e10".into(),
            sdp,
            ice_servers: vec![
                crate::IceServer {
                    urls: vec!["turn:turn.example.com:3478?transport=udp".into(); 4],
                    username: Some("u".repeat(200)),
                    credential: Some("c".repeat(200)),
                };
                4
            ],
            site_token: "t".repeat(2048),
        };
        let mut buffer = Vec::new();
        write_message(&mut buffer, &offer).await.unwrap();
        assert!(buffer.len() < MAX_FRAME_LEN as usize / 2);
    }
}
