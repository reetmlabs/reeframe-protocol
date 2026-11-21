use serde::{Deserialize, Serialize};

use crate::IceServer;

/// A control channel message, tagged by `type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    /// Sent by the backend to keep the connection alive.
    Ping,
    /// The Coordinator's answer to `ping`.
    Pong,
    /// Sent by the Coordinator right after a WebSocket connects. `nonce` is
    /// `NONCE_LEN` random bytes, base64url without padding.
    Challenge { nonce: String },
    /// The backend's answer to `challenge`. Both fields are base64url without
    /// padding: the Ed25519 public key, in the same form as at enrollment, and
    /// the Ed25519 signature over `auth_payload`.
    Auth {
        public_key: String,
        signature: String,
    },
    /// Sent by the Coordinator once `auth` checked out.
    AuthOk,
    /// Sent by the Coordinator to start a WebRTC session. `session_id` is a
    /// UUID the Coordinator picks, and `site_token` is a Coordinator-signed
    /// site JWT with `aud` set to the backend's id, which the backend verifies.
    SessionOffer {
        session_id: String,
        sdp: String,
        ice_servers: Vec<IceServer>,
        site_token: String,
    },
    /// The backend's answer to `session_offer`. The SDP already holds all of
    /// the backend's ICE candidates, so the backend sends no candidates itself.
    SessionAnswer { session_id: String, sdp: String },
    /// An ICE candidate of the client, relayed by the Coordinator while the
    /// client trickles them.
    SessionCandidate {
        session_id: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u32>,
    },
    /// Ends a session. Either side may send it.
    SessionClose { session_id: String, reason: String },
    /// Sent by the backend when it cannot start or keep a session. `code` is
    /// machine-readable, see `session_error`; `message` is for humans.
    SessionError {
        session_id: String,
        code: String,
        message: String,
    },
    /// Any type this version does not know. Ignored, so either side can add
    /// message types without breaking the other.
    #[serde(other)]
    Unknown,
}

impl Message {
    /// Encodes the message as one WebSocket text frame. WSS carries no length
    /// prefix, since the WebSocket frame already delimits the message.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("a message always serializes")
    }

    /// Decodes one WebSocket text frame. An unknown `type` gives
    /// [`Message::Unknown`], like on QUIC.
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Both sides parse this JSON, so a rename here would break deployed peers.
    #[test]
    fn ping_and_pong_have_a_fixed_wire_form() {
        assert_eq!(
            serde_json::to_string(&Message::Ping).unwrap(),
            r#"{"type":"ping"}"#
        );
        assert_eq!(
            serde_json::to_string(&Message::Pong).unwrap(),
            r#"{"type":"pong"}"#
        );
    }

    #[test]
    fn login_messages_have_a_fixed_wire_form() {
        assert_eq!(
            serde_json::to_string(&Message::Challenge {
                nonce: "n0nce".into()
            })
            .unwrap(),
            r#"{"type":"challenge","nonce":"n0nce"}"#
        );
        assert_eq!(
            serde_json::to_string(&Message::Auth {
                public_key: "pk".into(),
                signature: "sig".into()
            })
            .unwrap(),
            r#"{"type":"auth","public_key":"pk","signature":"sig"}"#
        );
        assert_eq!(
            serde_json::to_string(&Message::AuthOk).unwrap(),
            r#"{"type":"auth_ok"}"#
        );
    }

    #[test]
    fn json_helpers_round_trip_and_ignore_unknown_types() {
        let auth = Message::Auth {
            public_key: "pk".into(),
            signature: "sig".into(),
        };
        assert_eq!(Message::from_json(&auth.to_json()).unwrap(), auth);
        assert_eq!(
            Message::from_json(r#"{"type":"from_the_future","x":1}"#).unwrap(),
            Message::Unknown
        );
    }

    #[test]
    fn session_offer_has_a_fixed_wire_form() {
        let offer = Message::SessionOffer {
            session_id: "0b6f7c3e-4c1a-4f0e-9a51-2d3c1f6b8e10".into(),
            sdp: "v=0\r\n".into(),
            ice_servers: vec![
                IceServer {
                    urls: vec!["stun:stun.example.com:3478".into()],
                    username: None,
                    credential: None,
                },
                IceServer {
                    urls: vec!["turn:turn.example.com:3478".into()],
                    username: Some("user".into()),
                    credential: Some("secret".into()),
                },
            ],
            site_token: "jwt".into(),
        };
        assert_eq!(
            offer.to_json(),
            r#"{"type":"session_offer","session_id":"0b6f7c3e-4c1a-4f0e-9a51-2d3c1f6b8e10","sdp":"v=0\r\n","ice_servers":[{"urls":["stun:stun.example.com:3478"]},{"urls":["turn:turn.example.com:3478"],"username":"user","credential":"secret"}],"site_token":"jwt"}"#
        );
    }

    #[test]
    fn session_answer_has_a_fixed_wire_form() {
        let answer = Message::SessionAnswer {
            session_id: "s1".into(),
            sdp: "v=0\r\n".into(),
        };
        assert_eq!(
            answer.to_json(),
            r#"{"type":"session_answer","session_id":"s1","sdp":"v=0\r\n"}"#
        );
    }

    #[test]
    fn session_candidate_has_a_fixed_wire_form() {
        let candidate = Message::SessionCandidate {
            session_id: "s1".into(),
            candidate: "candidate:1 1 udp 2122260223 192.0.2.1 54400 typ host".into(),
            sdp_mid: Some("0".into()),
            sdp_mline_index: Some(0),
        };
        assert_eq!(
            candidate.to_json(),
            r#"{"type":"session_candidate","session_id":"s1","candidate":"candidate:1 1 udp 2122260223 192.0.2.1 54400 typ host","sdp_mid":"0","sdp_mline_index":0}"#
        );
    }

    #[test]
    fn session_candidate_without_mid_and_index_has_nulls() {
        let candidate = Message::SessionCandidate {
            session_id: "s1".into(),
            candidate: "candidate:1".into(),
            sdp_mid: None,
            sdp_mline_index: None,
        };
        let json = r#"{"type":"session_candidate","session_id":"s1","candidate":"candidate:1","sdp_mid":null,"sdp_mline_index":null}"#;
        assert_eq!(candidate.to_json(), json);
        // A peer that leaves the fields out means the same.
        assert_eq!(
            Message::from_json(
                r#"{"type":"session_candidate","session_id":"s1","candidate":"candidate:1"}"#
            )
            .unwrap(),
            candidate
        );
    }

    #[test]
    fn session_close_has_a_fixed_wire_form() {
        let close = Message::SessionClose {
            session_id: "s1".into(),
            reason: "client left".into(),
        };
        assert_eq!(
            close.to_json(),
            r#"{"type":"session_close","session_id":"s1","reason":"client left"}"#
        );
    }

    #[test]
    fn session_error_has_a_fixed_wire_form() {
        let error = Message::SessionError {
            session_id: "s1".into(),
            code: "too_many_sessions".into(),
            message: "limit of 4 reached".into(),
        };
        assert_eq!(
            error.to_json(),
            r#"{"type":"session_error","session_id":"s1","code":"too_many_sessions","message":"limit of 4 reached"}"#
        );
    }
}
