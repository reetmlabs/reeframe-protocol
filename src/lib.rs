//! Wire protocol of the control channel between the Reeframe backend and the
//! Coordinator. Both sides depend on this crate so they cannot drift apart.

mod close;
mod frame;
mod message;
mod session;
mod ws;

pub use close::CloseReason;
pub use frame::{read_message, write_message, MAX_FRAME_LEN};
pub use message::Message;
pub use session::{session_error, IceServer};
pub use ws::{auth_payload, NONCE_LEN, NONCE_TTL, WS_PATH};

/// ALPN protocol id of the control channel. Bump the version suffix only for
/// changes that old peers cannot handle.
pub const ALPN: &[u8] = b"reeframe-control/1";
