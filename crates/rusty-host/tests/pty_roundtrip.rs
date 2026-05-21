//! Integration tests for `PtySession` — real shells, headless. This is the core
//! de-risking for the Phase-1 PTY spike: it proves a shell spawns, output is
//! captured, resize works, and exit is detected, all without a GUI.
//!
//! Windows note: ConPTY withholds *all* output until the attached terminal answers
//! its startup cursor-position query (`ESC[6n`) with a CPR report (`ESC[row;colR`).
//! These tests include a minimal DSR responder so the shell can proceed; the real
//! app answers the same query with the grid's actual cursor position (renderer +
//! app wiring), so this mirrors production behaviour rather than working around it.

use std::time::{Duration, Instant};

use rusty_host::PtySession;

fn test_shell() -> String {
    if cfg!(windows) {
        "cmd.exe".to_string()
    } else {
        rusty_host::default_shell()
    }
}

/// Drain output until `needle` is seen or the deadline passes, answering any
/// `ESC[6n` cursor-position query with `ESC[1;1R` so ConPTY keeps producing output.
/// Returns the accumulated output.
fn drain_until(session: &mut PtySession, needle: &str, timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    let mut out = String::new();
    while Instant::now() < deadline {
        if let Some(chunk) = session.try_recv() {
            if chunk.windows(4).any(|w| w == b"\x1b[6n") {
                let _ = session.write(b"\x1b[1;1R");
            }
            out.push_str(&String::from_utf8_lossy(&chunk));
            if !needle.is_empty() && out.contains(needle) {
                break;
            }
        } else {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    out
}

/// Spawn a shell, run `echo`, and confirm the output reaches the receiver.
#[test]
fn test_pty_echo_roundtrip() {
    let cwd = std::env::temp_dir();
    let mut session = PtySession::spawn(&test_shell(), &cwd, 24, 80, || {}).expect("spawn shell");

    // Let the shell finish its startup handshake before typing.
    drain_until(&mut session, "", Duration::from_millis(800));
    session.write(b"echo rusty_marker\r").expect("write");

    let output = drain_until(&mut session, "rusty_marker", Duration::from_secs(15));
    assert!(
        output.contains("rusty_marker"),
        "expected echoed marker in shell output; got: {output:?}"
    );
}

/// Resizing the PTY succeeds.
#[test]
fn test_pty_resize_ok() {
    let cwd = std::env::temp_dir();
    let session = PtySession::spawn(&test_shell(), &cwd, 24, 80, || {}).expect("spawn shell");
    assert!(session.resize(40, 120).is_ok());
}

/// After `exit`, the session reports not-alive within the deadline.
#[test]
fn test_pty_exit_marks_not_alive() {
    let cwd = std::env::temp_dir();
    let mut session = PtySession::spawn(&test_shell(), &cwd, 24, 80, || {}).expect("spawn shell");

    drain_until(&mut session, "", Duration::from_millis(800));
    session.write(b"exit\r").expect("write");

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && session.is_alive() {
        if let Some(chunk) = session.try_recv() {
            if chunk.windows(4).any(|w| w == b"\x1b[6n") {
                let _ = session.write(b"\x1b[1;1R");
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    assert!(!session.is_alive(), "session still alive after `exit`");
}
