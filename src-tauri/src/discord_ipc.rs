// Shared Discord IPC framing, handshake and SET_ACTIVITY helpers.
// All three commands (check_discord_session, set_discord_activity,
// spoof_non_exe_quest) previously duplicated this logic inline.
//
// Protocol (Discord IPC):
//   frame := op(u32 LE) | payload_len(u32 LE) | payload
//   op 0 = HANDSHAKE, op 1 = FRAME

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
/// callers work with either through a single boxed value.
pub trait ReadWrite: Read + Write {}

impl<T: Read + Write> ReadWrite for T {}

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

/// Perform the Discord IPC handshake (op 0) and return the parsed response.
/// The response contains the logged-in user under `data.user`.
pub fn handshake(stream: &mut dyn ReadWrite, client_id: &str) -> io::Result<serde_json::Value> {
    let payload = format!(r#"{{"v":1,"client_id":"{}"}}"#, client_id);
    send_frame(stream, 0, payload.as_bytes())?;
    let (_, resp) = decode_frame(stream)?;
    serde_json::from_slice(&resp)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
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
    let payload = format!(
        r#"{{"cmd":"SET_ACTIVITY","args":{{"pid":{},"activity":{{"details":"{}","state":"{}","timestamps":{{"start":{},"end":{}}}}}}},"nonce":"{}"}}"#,
        pid, details, state, start_ts, end_ts, nonce
    );
    send_frame(stream, 1, payload.as_bytes())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

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
}
