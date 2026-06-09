//! LSP JSON-RPC message framing and communication.
//!
//! Spawns rust-analyzer and manages async input/output JSON-RPC message serialization.

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use serde_json::{json, Value};

/// Write a JSON-RPC message with Content-Length header to a writer.
pub fn write_message<W: Write>(writer: &mut W, message: &Value) -> io::Result<()> {
    let payload = serde_json::to_string(message)?;
    println!("[lsp-send] {:?}", message);
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

/// Helper to convert a filesystem path to a normalized lowercase drive letter file URI.
pub fn path_to_uri(path: &Path) -> Result<lsp_types::Uri, String> {
    let mut path_str = path.to_string_lossy().to_string();
    path_str = path_str.replace('\\', "/");

    let uri_str = if path_str.starts_with('/') {
        format!("file://{}", path_str)
    } else {
        if path_str.len() > 2 && &path_str[1..3] == ":/" {
            let drive = path_str.as_bytes()[0].to_ascii_lowercase() as char;
            format!("file:///{}:{}", drive, &path_str[2..])
        } else {
            format!("file:///{}", path_str)
        }
    };

    lsp_types::Uri::from_str(&uri_str)
        .map_err(|e| format!("Failed to parse Uri '{uri_str}': {e}"))
}

/// An active LSP session managing a `rust-analyzer` child process.
pub struct LspSession {
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    next_id: AtomicU64,
    pending_requests: Arc<Mutex<HashMap<u64, Sender<Value>>>>,
    diag_rx: Receiver<lsp_types::PublishDiagnosticsParams>,
}

impl LspSession {
    /// Spawn rust-analyzer in `root_path` and perform the initialization handshake.
    pub fn new(root_path: &Path) -> Result<Self, String> {
        let mut child = Command::new("rust-analyzer")
            .current_dir(root_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn rust-analyzer: {e}"))?;

        let stdin = Arc::new(Mutex::new(child.stdin.take().ok_or("Failed to open stdin")?));
        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to open stderr")?;

        // Spawn stderr printer
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(l) = line {
                    eprintln!("[lsp-stderr] {}", l);
                }
            }
        });


        // Perform initialization handshake synchronously
        let root_uri = path_to_uri(root_path)?;
        let mut params = lsp_types::InitializeParams::default();
        #[allow(deprecated)]
        {
            params.root_uri = Some(root_uri);
        }

        let mut text_document = lsp_types::TextDocumentClientCapabilities::default();

        let mut synchronization = lsp_types::TextDocumentSyncClientCapabilities::default();
        synchronization.dynamic_registration = Some(true);
        synchronization.will_save = Some(true);
        synchronization.did_save = Some(true);
        text_document.synchronization = Some(synchronization);

        let mut hover = lsp_types::HoverClientCapabilities::default();
        hover.content_format = Some(vec![lsp_types::MarkupKind::Markdown, lsp_types::MarkupKind::PlainText]);
        text_document.hover = Some(hover);

        let mut completion = lsp_types::CompletionClientCapabilities::default();
        let mut completion_item = lsp_types::CompletionItemCapability::default();
        completion_item.snippet_support = Some(false);
        completion.completion_item = Some(completion_item);
        text_document.completion = Some(completion);

        let mut publish_diagnostics = lsp_types::PublishDiagnosticsClientCapabilities::default();
        publish_diagnostics.version_support = Some(true);
        text_document.publish_diagnostics = Some(publish_diagnostics);

        params.capabilities.text_document = Some(text_document);

        let init_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": params
        });

        write_message(&mut *stdin.lock().unwrap(), &init_request)
            .map_err(|e| format!("Failed to write initialize request: {e}"))?;

        let mut reader = BufReader::new(stdout);
        loop {
            match read_message(&mut reader) {
                Ok(Some(msg)) => {
                    if msg.get("id").and_then(|v| v.as_u64()) == Some(1) {
                        println!("[lsp-init-res] {:?}", msg);
                        break; // Handshake accepted
                    }
                }
                Ok(None) => return Err("rust-analyzer exited during initialization".to_string()),
                Err(e) => return Err(format!("Error during initialization: {e}")),
            }
        }

        let init_notification = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });

        write_message(&mut *stdin.lock().unwrap(), &init_notification)
            .map_err(|e| format!("Failed to write initialized notification: {e}"))?;

        let pending_requests: Arc<Mutex<HashMap<u64, Sender<Value>>>> = Arc::new(Mutex::new(HashMap::new()));
        let (diag_tx, diag_rx) = channel();

        // Spawn background reader thread
        let pending_requests_clone = Arc::clone(&pending_requests);
        let stdin_clone = Arc::clone(&stdin);
        std::thread::spawn(move || {
            loop {
                match read_message(&mut reader) {
                    Ok(Some(msg)) => {
                        println!("[lsp-recv] {:?}", msg);
                        if let Some(id_val) = msg.get("id") {
                            if let Some(method) = msg.get("method").and_then(|v| v.as_str()) {
                                // Request from the server! Respond.
                                let response = match method {
                                    "workspace/configuration" => {
                                        let items_len = msg.get("params")
                                            .and_then(|p| p.get("items"))
                                            .and_then(|i| i.as_array())
                                            .map(|a| a.len())
                                            .unwrap_or(1);
                                        json!({
                                            "jsonrpc": "2.0",
                                            "id": id_val,
                                            "result": vec![Value::Null; items_len]
                                        })
                                    }
                                    _ => {
                                        json!({
                                            "jsonrpc": "2.0",
                                            "id": id_val,
                                            "result": Value::Null
                                        })
                                    }
                                };
                                let mut stdin_lock = stdin_clone.lock().unwrap();
                                let _ = write_message(&mut *stdin_lock, &response);
                            } else {
                                // Response from the server to one of our requests
                                if let Some(id) = id_val.as_u64() {
                                    let mut pending = pending_requests_clone.lock().unwrap();
                                    if let Some(tx) = pending.remove(&id) {
                                        let _ = tx.send(msg);
                                    }
                                }
                            }
                        } else {
                            // Notification from the server (no id)
                            if let Some(method) = msg.get("method").and_then(|v| v.as_str()) {
                                if method == "textDocument/publishDiagnostics" {
                                    if let Some(params) = msg.get("params") {
                                        if let Ok(diag_params) = serde_json::from_value::<lsp_types::PublishDiagnosticsParams>(params.clone()) {
                                            let _ = diag_tx.send(diag_params);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(None) => break, // EOF
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            next_id: AtomicU64::new(2),
            pending_requests,
            diag_rx,
        })
    }

    /// Notify the server that a document was opened.
    pub fn did_open(&self, path: &Path, text: &str) -> Result<(), String> {
        let uri = path_to_uri(path)?;
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "rust",
                    "version": 1,
                    "text": text
                }
            }
        });
        write_message(&mut *self.stdin.lock().unwrap(), &notification)
            .map_err(|e| format!("Failed to send didOpen: {e}"))
    }

    /// Notify the server that a document was modified.
    pub fn did_change(&self, path: &Path, version: i32, text: &str) -> Result<(), String> {
        let uri = path_to_uri(path)?;
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "version": version
                },
                "contentChanges": [
                    {
                        "text": text
                    }
                ]
            }
        });
        write_message(&mut *self.stdin.lock().unwrap(), &notification)
            .map_err(|e| format!("Failed to send didChange: {e}"))
    }

    /// Query hover info at a position in the document.
    pub fn hover(&self, path: &Path, line: u32, character: u32) -> Result<Receiver<Result<Option<lsp_types::Hover>, String>>, String> {
        let uri = path_to_uri(path)?;
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = channel();

        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.insert(id, tx);
        }

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": uri
                },
                "position": {
                    "line": line,
                    "character": character
                }
            }
        });

        write_message(&mut *self.stdin.lock().unwrap(), &request)
            .map_err(|e| format!("Failed to send hover request: {e}"))?;

        let (out_tx, out_rx) = channel();
        std::thread::spawn(move || {
            match rx.recv() {
                Ok(resp) => {
                    if let Some(err) = resp.get("error") {
                        let _ = out_tx.send(Err(format!("LSP Hover Error: {err:?}")));
                    } else {
                        let result = resp.get("result").cloned().unwrap_or(Value::Null);
                        if result.is_null() {
                            let _ = out_tx.send(Ok(None));
                        } else {
                            match serde_json::from_value::<lsp_types::Hover>(result) {
                                Ok(hover) => {
                                    let _ = out_tx.send(Ok(Some(hover)));
                                }
                                Err(e) => {
                                    let _ = out_tx.send(Err(format!("Failed to parse hover: {e}")));
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    let _ = out_tx.send(Err("LSP session disconnected".to_string()));
                }
            }
        });

        Ok(out_rx)
    }

    /// Query autocomplete completions at a position.
    pub fn completion(&self, path: &Path, line: u32, character: u32) -> Result<Receiver<Result<Vec<lsp_types::CompletionItem>, String>>, String> {
        let uri = path_to_uri(path)?;
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = channel();

        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.insert(id, tx);
        }

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": uri
                },
                "position": {
                    "line": line,
                    "character": character
                },
                "context": {
                    "triggerKind": 2, // TriggerCharacter
                    "triggerCharacter": "."
                }
            }
        });

        write_message(&mut *self.stdin.lock().unwrap(), &request)
            .map_err(|e| format!("Failed to send completion request: {e}"))?;

        let (out_tx, out_rx) = channel();
        std::thread::spawn(move || {
            match rx.recv() {
                Ok(resp) => {
                    if let Some(err) = resp.get("error") {
                        let _ = out_tx.send(Err(format!("LSP Completion Error: {err:?}")));
                    } else {
                        let result = resp.get("result").cloned().unwrap_or(Value::Null);
                        if result.is_null() {
                            let _ = out_tx.send(Ok(Vec::new()));
                        } else {
                            let items = if let Ok(list) = serde_json::from_value::<lsp_types::CompletionResponse>(result.clone()) {
                                match list {
                                    lsp_types::CompletionResponse::Array(items) => items,
                                    lsp_types::CompletionResponse::List(completion_list) => completion_list.items,
                                }
                            } else if let Ok(items) = serde_json::from_value::<Vec<lsp_types::CompletionItem>>(result) {
                                items
                            } else {
                                Vec::new()
                            };
                            let _ = out_tx.send(Ok(items));
                        }
                    }
                }
                Err(_) => {
                    let _ = out_tx.send(Err("LSP session disconnected".to_string()));
                }
            }
        });

        Ok(out_rx)
    }

    /// Notify the server that a document was saved.
    pub fn did_save(&self, path: &Path) -> Result<(), String> {
        let uri = path_to_uri(path)?;
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didSave",
            "params": {
                "textDocument": {
                    "uri": uri
                }
            }
        });
        write_message(&mut *self.stdin.lock().unwrap(), &notification)
            .map_err(|e| format!("Failed to send didSave: {e}"))
    }

    /// Poll diagnostics published by the server.
    pub fn poll_diagnostics(&self) -> Option<lsp_types::PublishDiagnosticsParams> {
        self.diag_rx.try_recv().ok()
    }
}

impl Drop for LspSession {
    fn drop(&mut self) {
        // Send shutdown request
        let shutdown = json!({
            "jsonrpc": "2.0",
            "id": 999,
            "method": "shutdown",
            "params": null
        });
        if write_message(&mut *self.stdin.lock().unwrap(), &shutdown).is_ok() {
            let exit = json!({
                "jsonrpc": "2.0",
                "method": "exit",
                "params": null
            });
            let _ = write_message(&mut *self.stdin.lock().unwrap(), &exit);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
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
