//! Login on the WebSocket (WSS) transport of the control channel. The
//! Coordinator sends `challenge`, the backend signs `auth_payload` and
//! answers with `auth`, and the Coordinator confirms with `auth_ok`.

use std::time::Duration;

/// HTTP path of the control channel WebSocket on the Coordinator.
pub const WS_PATH: &str = "/control/ws";

/// Number of random bytes in a challenge nonce, before base64url encoding.
pub const NONCE_LEN: usize = 32;

/// How long a nonce stays valid. A nonce is accepted once and only within
/// this time, so a captured `auth` cannot be replayed.
pub const NONCE_TTL: Duration = Duration::from_secs(10);

/// The bytes the backend signs to answer a challenge:
/// `reeframe-control-v1\n{nonce}\n{host}`. The fixed prefix separates this
/// signature from the enrollment signature, and the host binds it to one
/// Coordinator.
///
/// `host` is the Coordinator hostname the client connected to. This function
/// lowercases it; stripping the port is the caller's job, so pass
/// `coordinator.example.com`, not `coordinator.example.com:443`.
pub fn auth_payload(nonce: &str, host: &str) -> Vec<u8> {
    format!("reeframe-control-v1\n{nonce}\n{}", host.to_lowercase()).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Both sides compute these bytes independently, so they must never change.
    #[test]
    fn the_auth_payload_has_fixed_bytes() {
        assert_eq!(
            auth_payload("n0nce", "coordinator.example.com"),
            b"reeframe-control-v1\nn0nce\ncoordinator.example.com"
        );
    }

    #[test]
    fn the_auth_payload_lowercases_the_host_but_keeps_the_nonce() {
        assert_eq!(
            auth_payload("AbC_-x", "Coordinator.EXAMPLE.com"),
            b"reeframe-control-v1\nAbC_-x\ncoordinator.example.com"
        );
    }

    #[test]
    fn the_auth_payload_takes_the_host_as_given() {
        // The port is not stripped here; callers pass the bare hostname.
        assert_eq!(
            auth_payload("n", "host:443"),
            b"reeframe-control-v1\nn\nhost:443"
        );
    }
}
