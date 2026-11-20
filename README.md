# reeframe-protocol

Wire protocol of the control channel between the Reeframe backend and the
Reeframe Coordinator. Both sides depend on this crate, so the message types,
the framing and the shared constants live in one place.

## Frame format

Each message is one frame on a QUIC stream:

```
+----------------------+---------------------+
| length (u32, BE)     | JSON body           |
| 4 bytes              | `length` bytes      |
+----------------------+---------------------+
```

- The length is a 4-byte big-endian integer and counts only the body.
- A frame larger than `MAX_FRAME_LEN` (64 KiB) is refused.
- The body is a JSON object tagged by `type`, for example `{"type":"ping"}`.
- A clean end of stream before a frame starts reads as `None`.

The connection uses the ALPN id `reeframe-control/1`. When the Coordinator
closes a connection, it sends a `CloseReason` as the QUIC application close
code (0 closed, 1 unknown key, 2 replaced, 3 timeout, 4 revoked, 5 protocol).

## Usage

```toml
[dependencies]
reeframe-protocol = { git = "https://github.com/reetmlabs/reeframe-protocol", tag = "v0.1.0" }
```

```rust
use reeframe_protocol::{read_message, write_message, Message};

write_message(&mut send, &Message::Ping).await?;
if let Some(Message::Pong) = read_message(&mut recv).await? {
    // the Coordinator is alive
}
```

## Compatibility

Adding a message type is backwards compatible. A peer reads any `type` it does
not know as `Message::Unknown` and ignores it, so either side can ship a new
message before the other knows it. Renaming or removing a type, or changing
the frame format, is a breaking change and needs a new ALPN id.

## License

Apache-2.0, see [LICENSE](LICENSE).
