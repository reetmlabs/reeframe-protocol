//! Types for signaling WebRTC sessions over the control channel.

use serde::{Deserialize, Serialize};

/// A STUN or TURN server for one session, in the shape of WebRTC's
/// `RTCIceServer`. Missing credentials are left out of the JSON so a STUN
/// server reads as just its URLs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

/// Known `code` values of `Message::SessionError`. The field stays a string,
/// so a newer backend can send codes this version does not list.
pub mod session_error {
    /// The backend already runs as many sessions as it allows.
    pub const TOO_MANY_SESSIONS: &str = "too_many_sessions";
    /// The `site_token` of the offer did not verify.
    pub const INVALID_TOKEN: &str = "invalid_token";
    /// The SDP of the offer could not be used.
    pub const INVALID_OFFER: &str = "invalid_offer";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ice_server_without_credentials_has_only_urls() {
        let server = IceServer {
            urls: vec!["stun:stun.example.com:3478".into()],
            username: None,
            credential: None,
        };
        let json = r#"{"urls":["stun:stun.example.com:3478"]}"#;
        assert_eq!(serde_json::to_string(&server).unwrap(), json);
        assert_eq!(serde_json::from_str::<IceServer>(json).unwrap(), server);
    }

    #[test]
    fn an_ice_server_with_credentials_has_a_fixed_wire_form() {
        let server = IceServer {
            urls: vec![
                "turn:turn.example.com:3478?transport=udp".into(),
                "turns:turn.example.com:5349".into(),
            ],
            username: Some("user".into()),
            credential: Some("secret".into()),
        };
        let json = r#"{"urls":["turn:turn.example.com:3478?transport=udp","turns:turn.example.com:5349"],"username":"user","credential":"secret"}"#;
        assert_eq!(serde_json::to_string(&server).unwrap(), json);
        assert_eq!(serde_json::from_str::<IceServer>(json).unwrap(), server);
    }
}
