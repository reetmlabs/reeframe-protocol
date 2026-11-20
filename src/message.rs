use serde::{Deserialize, Serialize};

/// A control channel message, tagged by `type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    /// Sent by the backend to keep the connection alive.
    Ping,
    /// The Coordinator's answer to `ping`.
    Pong,
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
}
