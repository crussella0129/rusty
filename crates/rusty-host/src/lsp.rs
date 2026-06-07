//! LSP JSON-RPC message framing and communication.
//!
//! Spawns rust-analyzer and manages async input/output JSON-RPC message serialization.

use std::io::{self, BufRead, BufReader, Read, Write};
use serde_json::Value;

/// Write a JSON-RPC message with Content-Length header to a writer.
pub fn write_message<W: Write>(writer: &mut W, message: &Value) -> io::Result<()> {
    let payload = serde_json::to_string(message)?;
    write!(writer, "Content-Length: {}\r\n\r\n{}", payload.len(), payload)?;
    writer.flush()?;
    Ok(())
}

/// Read a JSON-RPC message with Content-Length header from a buffered reader.
pub fn read_message<R: Read>(reader: &mut BufReader<R>) -> io::Result<Option<Value>> {
    let mut line = String::new();
    let mut content_length = None;

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            return Ok(None); // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            break; // Header section ended, JSON payload follows
        }

        // Case-insensitive check just in case, though standard is "Content-Length:"
        if trimmed.to_lowercase().starts_with("content-length:") {
            if let Some(rest) = trimmed.split(':').nth(1) {
                if let Ok(len) = rest.trim().parse::<usize>() {
                    content_length = Some(len);
                }
            }
        }
    }

    let Some(len) = content_length else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Missing Content-Length header",
        ));
    };

    let mut payload = vec![0; len];
    reader.read_exact(&mut payload)?;

    let val = serde_json::from_slice(&payload)?;
    Ok(Some(val))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;

    #[test]
    fn test_jsonrpc_framing_read_and_write() {
        let original_msg = json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "test",
            "params": {"ok": true}
        });

        let mut buffer = Vec::new();
        write_message(&mut buffer, &original_msg).unwrap();

        let expected_prefix = format!("Content-Length: {}\r\n\r\n", serde_json::to_string(&original_msg).unwrap().len());
        assert!(buffer.starts_with(expected_prefix.as_bytes()));

        let mut reader = BufReader::new(Cursor::new(buffer));
        let read_msg = read_message(&mut reader).unwrap().unwrap();

        assert_eq!(original_msg, read_msg);
    }
}
