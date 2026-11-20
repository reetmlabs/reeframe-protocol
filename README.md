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

## WebSocket (WSS)

The control channel can also run over a WebSocket at `WS_PATH`
(`/control/ws`) on the Coordinator. Each message is one JSON text frame, with
no length prefix. Use `Message::to_json` and `Message::from_json` so both
sides encode the same way.

Login sequence:

1. Coordinator -> backend: `{"type":"challenge","nonce":"..."}`. The nonce is
   `NONCE_LEN` (32) random bytes, base64url without padding. It is valid once
   and for `NONCE_TTL` (10 s).
2. Backend -> Coordinator:
   `{"type":"auth","public_key":"...","signature":"..."}`. The public key is
   the backend's Ed25519 key, in the same form as at enrollment, and the
   signature is an Ed25519 signature over `auth_payload(nonce, host)`. Both
   are base64url without padding.
3. Coordinator -> backend: `{"type":"auth_ok"}`. After that, the usual
   messages (`ping`, `pong`, ...) follow.

`auth_payload` returns the bytes `reeframe-control-v1\n{nonce}\n{host}`,
where `host` is the Coordinator hostname the backend connected to, lowercased
and without port. The function lowercases; the caller strips the port.

On WebSocket the close code is `4000 + n` for the `CloseReason` code `n`
(4000 closed, 4001 unknown key, 4002 replaced, 4003 timeout, 4004 revoked,
4005 protocol), see `CloseReason::ws_code` and `CloseReason::from_ws_code`.

This crate has no crypto dependencies. Signing and verifying happen in the
backend and the Coordinator.

## Usage

```toml
[dependencies]
reeframe-protocol = { git = "https://github.com/reetmlabs/reeframe-protocol", tag = "v0.2.0" }
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
