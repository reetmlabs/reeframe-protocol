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

## WebRTC sessions

The control channel also carries the signaling of WebRTC sessions between a
client and the backend, relayed by the Coordinator. Every session message
names its session by `session_id`, a UUID the Coordinator picks.

| Message | Direction | Fields |
|---|---|---|
| `session_offer` | Coordinator -> backend | `session_id`, `sdp`, `ice_servers`, `site_token` |
| `session_answer` | backend -> Coordinator | `session_id`, `sdp` |
| `session_candidate` | Coordinator -> backend | `session_id`, `candidate`, `sdp_mid`, `sdp_mline_index` |
| `session_close` | either | `session_id`, `reason` |
| `session_error` | backend -> Coordinator | `session_id`, `code`, `message` |

Flow:

1. The Coordinator sends `session_offer` with the client's SDP offer, the
   STUN/TURN servers to use, and `site_token`, a Coordinator-signed site JWT
   with `aud` set to the backend's id. The backend verifies the token.
2. The backend answers with `session_answer`. Its SDP already holds all of
   the backend's ICE candidates; the backend does not trickle.
3. While the client trickles, the Coordinator relays each client candidate
   as `session_candidate`. `sdp_mid` and `sdp_mline_index` may be `null`.
4. Either side ends the session with `session_close`. If the backend cannot
   start or keep a session, it sends `session_error` instead, with a `code`
   such as `too_many_sessions`, `invalid_token` or `invalid_offer` (see
   `session_error::*`). New codes may appear, so treat unknown codes as a
   generic failure.

`ice_servers` entries look like WebRTC's `RTCIceServer`:
`{"urls":["turn:turn.example.com:3478"],"username":"...","credential":"..."}`.
`username` and `credential` are left out when not set.

Size: a session offer must fit in one QUIC frame of `MAX_FRAME_LEN` (64 KiB).
An SDP is usually under 10 KiB. JSON escapes each CRLF line end as `\r\n`,
which adds 2 bytes per SDP line. A test checks that an offer with a 22 KiB
SDP, four TURN servers and a 2 KiB token stays under 32 KiB.

## Usage

```toml
[dependencies]
reeframe-protocol = { git = "https://github.com/reetmlabs/reeframe-protocol", tag = "v0.3.0" }
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
