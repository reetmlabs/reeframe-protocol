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
