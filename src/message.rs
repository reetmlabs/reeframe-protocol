use serde::{Deserialize, Serialize};

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
    /// Any type this version does not know. Ignored, so either side can add
    /// message types without breaking the other.
    #[serde(other)]
    Unknown,
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
}
