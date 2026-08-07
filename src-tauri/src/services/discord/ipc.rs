//! Shared Discord IPC framing, handshake and SET_ACTIVITY helpers.
//!
//! Protocol (Discord IPC):
//!   frame := op(u32 LE) | payload_len(u32 LE) | payload
//!   op 0 = HANDSHAKE, op 1 = FRAME
//!
//! Payloads are built with `serde_json::json!` so user-controlled strings
//! (game titles containing quotes, emoji, newlines) are always escaped —
//! the old `format!`-assembled JSON could emit invalid frames for such names.

use std::io::{self, Read, Write};
use std::path::PathBuf;

/// Upper bound for a single IPC payload. Guards against allocating on a
/// corrupted/hostile length field (previously `vec![0; len]` on an unbounded
/// u32 could exhaust memory).
const MAX_PAYLOAD_LEN: usize = 1 << 20;

/// Windows named pipe that the Discord desktop client exposes.
#[cfg(target_os = "windows")]
pub const PIPE_PATH: &str = r"\\.\pipe\discord-ipc-0";

/// Trait object for duplex IPC byte streams.
///
/// Both the Windows named pipe (`std::fs::File`) and the Unix domain socket
/// (`std::os::unix::net::UnixStream`) expose `Read + Write`; this trait lets
/// callers work with either through a single boxed value. `Send` is required
/// so connections can cross `spawn_blocking`/tokio await boundaries.
pub trait ReadWrite: Read + Write + Send {}

impl<T: Read + Write + Send> ReadWrite for T {}

/// Typed result of a successful handshake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakeResult {
    pub username: String,
    pub user_id: String,
}

/// Candidate paths for the Discord IPC Unix domain socket, in priority order.
fn unix_socket_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    #[cfg(target_os = "macos")]
    {
        candidates.push(PathBuf::from("/tmp/discord-ipc-0"));
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
            candidates.push(PathBuf::from(dir).join("discord-ipc-0"));
        }
        candidates.push(PathBuf::from("/tmp/discord-ipc-0"));
    }
    candidates
}

/// Resolve the Discord IPC Unix socket path.
///
/// Returns the first candidate that already exists so we pick up whichever
/// socket Discord created; falls back to the primary candidate so connection
/// errors stay informative.
pub fn unix_socket_path() -> PathBuf {
    let candidates = unix_socket_candidates();
    candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| candidates[0].clone())
}

/// Open a fresh connection to the local Discord client.
#[cfg(target_os = "windows")]
pub fn open() -> io::Result<Box<dyn ReadWrite>> {
    use std::fs::OpenOptions;
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(PIPE_PATH)
        .map(|stream| Box::new(stream) as Box<dyn ReadWrite>)
}

/// Open a fresh connection to the local Discord client.
#[cfg(not(target_os = "windows"))]
pub fn open() -> io::Result<Box<dyn ReadWrite>> {
    use std::os::unix::net::UnixStream;
    let path = unix_socket_path();
    UnixStream::connect(&path).map(|stream| Box::new(stream) as Box<dyn ReadWrite>)
}

/// Serialize an IPC frame into a byte buffer.
pub fn encode_frame(op: u32, payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(8 + payload.len());
    buf.extend_from_slice(&op.to_le_bytes());
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(payload);
    buf
}

/// Read one IPC frame (op + payload) from `reader`.
pub fn decode_frame<R: Read + ?Sized>(reader: &mut R) -> io::Result<(u32, Vec<u8>)> {
    let mut op_buf = [0u8; 4];
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut op_buf)?;
    reader.read_exact(&mut len_buf)?;

    let op = u32::from_le_bytes(op_buf);
    let len = u32::from_le_bytes(len_buf) as usize;

    if len > MAX_PAYLOAD_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("IPC frame payload too large: {} bytes", len),
        ));
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload)?;
    Ok((op, payload))
}

/// Write one IPC frame to `writer`.
pub fn send_frame<W: Write + ?Sized>(writer: &mut W, op: u32, payload: &[u8]) -> io::Result<()> {
    writer.write_all(&encode_frame(op, payload))
}

fn io_serde(e: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e.to_string())
}

/// Build the HANDSHAKE payload (op 0).
fn handshake_payload(client_id: &str) -> serde_json::Value {
    serde_json::json!({ "v": 1, "client_id": client_id })
}

/// Perform the Discord IPC handshake (op 0) and return the logged-in user.
pub fn handshake(stream: &mut dyn ReadWrite, client_id: &str) -> io::Result<HandshakeResult> {
    let bytes = serde_json::to_vec(&handshake_payload(client_id)).map_err(io_serde)?;
    send_frame(stream, 0, &bytes)?;
    let (_, resp) = decode_frame(stream)?;
    let value: serde_json::Value = serde_json::from_slice(&resp).map_err(io_serde)?;

    let user = value.get("data").and_then(|d| d.get("user"));
    let username = user
        .and_then(|u| u.get("username"))
        .and_then(|u| u.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let user_id = user
        .and_then(|u| u.get("id"))
        .and_then(|i| i.as_str())
        .unwrap_or("")
        .to_string();

    Ok(HandshakeResult { username, user_id })
}

/// Build the SET_ACTIVITY payload (op 1) with a progress window.
fn activity_payload(
    pid: u32,
    details: &str,
    state: &str,
    start_ts: u64,
    end_ts: u64,
    nonce: &str,
) -> serde_json::Value {
    serde_json::json!({
        "cmd": "SET_ACTIVITY",
        "args": {
            "pid": pid,
            "activity": {
                "details": details,
                "state": state,
                "timestamps": { "start": start_ts, "end": end_ts }
            }
        },
        "nonce": nonce
    })
}

/// Send a SET_ACTIVITY (op 1) frame carrying Rich Presence with a progress
/// window defined by [start_ts, end_ts].
pub fn set_activity(
    stream: &mut dyn ReadWrite,
    pid: u32,
    details: &str,
    state: &str,
    start_ts: u64,
    end_ts: u64,
    nonce: &str,
) -> io::Result<()> {
    let payload = activity_payload(pid, details, state, start_ts, end_ts, nonce);
    let bytes = serde_json::to_vec(&payload).map_err(io_serde)?;
    send_frame(stream, 1, &bytes)
}

/// Send a CLEAR_ACTIVITY (op 1) frame, removing the app's Rich Presence.
pub fn clear_activity(stream: &mut dyn ReadWrite, pid: u32, nonce: &str) -> io::Result<()> {
    let payload = serde_json::json!({
        "cmd": "CLEAR_ACTIVITY",
        "args": { "pid": pid },
        "nonce": nonce,
    });
    let bytes = serde_json::to_vec(&payload).map_err(io_serde)?;
    send_frame(stream, 1, &bytes)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    struct TestStream {
        inp: Cursor<Vec<u8>>,
        out: Vec<u8>,
    }

    impl TestStream {
        fn from_response(frame: Vec<u8>) -> Self {
            TestStream {
                inp: Cursor::new(frame),
                out: Vec::new(),
            }
        }
    }

    impl Read for TestStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.inp.read(buf)
        }
    }

    impl Write for TestStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.out.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn encode_frame_layout() {
        let frame = encode_frame(1, b"hello");
        assert_eq!(frame.len(), 8 + 5);
        assert_eq!(&frame[0..4], &1u32.to_le_bytes());
        assert_eq!(&frame[4..8], &5u32.to_le_bytes());
        assert_eq!(&frame[8..], b"hello");
    }

    #[test]
    fn frame_round_trip() {
        let payload = b"{\"cmd\":\"SET_ACTIVITY\"}";
        let frame = encode_frame(1, payload);
        let mut cursor = Cursor::new(frame);
        let (op, data) = decode_frame(&mut cursor).unwrap();
        assert_eq!(op, 1);
        assert_eq!(data, payload);
    }

    #[test]
    fn decode_empty_payload() {
        let frame = encode_frame(0, b"");
        let mut cursor = Cursor::new(frame);
        let (op, data) = decode_frame(&mut cursor).unwrap();
        assert_eq!(op, 0);
        assert!(data.is_empty());
    }

    #[test]
    fn decode_rejects_oversized_payload() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.extend_from_slice(&u32::MAX.to_le_bytes());
        let mut cursor = Cursor::new(buf);
        assert!(decode_frame(&mut cursor).is_err());
    }

    #[test]
    fn decode_rejects_truncated_header() {
        let mut cursor = Cursor::new(vec![0u8, 1, 2]);
        assert!(decode_frame(&mut cursor).is_err());
    }

    #[test]
    fn handshake_payload_shape() {
        let payload = handshake_payload("12345");
        assert_eq!(payload["v"], 1);
        assert_eq!(payload["client_id"], "12345");
        // payload must serialize back to valid JSON
        let bytes = serde_json::to_vec(&payload).unwrap();
        assert!(serde_json::from_slice::<serde_json::Value>(&bytes).is_ok());
    }

    #[test]
    fn activity_payload_shape_and_escaping() {
        let details = r#"Endfield "Star" Expansion"#;
        let state = "Earning 700 Orbs\n★";
        let payload = activity_payload(42, details, state, 100, 900, "astral_1");
        assert_eq!(payload["cmd"], "SET_ACTIVITY");
        assert_eq!(payload["args"]["pid"], 42);
        assert_eq!(payload["args"]["activity"]["details"], details);
        assert_eq!(payload["args"]["activity"]["state"], state);
        assert_eq!(payload["args"]["activity"]["timestamps"]["start"], 100);
        assert_eq!(payload["args"]["activity"]["timestamps"]["end"], 900);
        assert_eq!(payload["nonce"], "astral_1");

        // Round-trips as valid JSON even with quotes/newlines in the strings.
        let bytes = serde_json::to_vec(&payload).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed["args"]["activity"]["details"], details);
    }

    #[test]
    fn handshake_parses_user() {
        let resp = serde_json::json!({
            "cmd": "HANDSHAKE",
            "evt": null,
            "data": {
                "v": 1,
                "config": { "cdn_host": "cdn.discordapp.com", "api_endpoint": "//discord.com/api" },
                "user": { "id": "24022911", "username": "thedev", "discriminator": "0000", "avatar": null }
            },
            "nonce": null
        });
        let frame = encode_frame(0, &serde_json::to_vec(&resp).unwrap());
        let mut stream = TestStream::from_response(frame);

        let result = handshake(&mut stream, "12345").unwrap();
        assert_eq!(result.username, "thedev");
        assert_eq!(result.user_id, "24022911");

        // The outgoing HANDSHAKE frame carries v=1 + the right client_id.
        let (op, payload) = decode_frame(&mut Cursor::new(stream.out)).unwrap();
        assert_eq!(op, 0);
        let sent: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert_eq!(sent["v"], 1);
        assert_eq!(sent["client_id"], "12345");
    }

    #[test]
    fn handshake_without_user_falls_back_to_unknown() {
        let resp = serde_json::json!({ "cmd": "HANDSHAKE", "data": { "v": 1 } });
        let frame = encode_frame(0, &serde_json::to_vec(&resp).unwrap());
        let mut stream = TestStream::from_response(frame);

        let result = handshake(&mut stream, "12345").unwrap();
        assert_eq!(result.username, "Unknown");
        assert_eq!(result.user_id, "");
    }
}
