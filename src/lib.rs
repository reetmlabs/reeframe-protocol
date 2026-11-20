//! Wire protocol of the control channel between the Reeframe backend and the
//! Coordinator. Both sides depend on this crate so they cannot drift apart.

mod frame;
mod message;

pub use frame::{read_message, write_message, MAX_FRAME_LEN};
pub use message::Message;
