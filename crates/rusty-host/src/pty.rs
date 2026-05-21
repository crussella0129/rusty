//! `PtySession` — a real pseudo-terminal running the learner's shell.
//!
//! This is the first of Rusty's three subprocesses (prompt §2). A platform shell is
//! launched in a PTY with its `cwd` locked to a sandbox directory; a background
//! reader thread streams the shell's output to the UI via an `mpsc` channel and a
//! repaint callback, while the UI writes keystrokes back through the master writer.
//!
//! All OS/subprocess coupling lives here (portability tripwire §11).

use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread;

use anyhow::Result;
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};

/// A live shell session attached to a pseudo-terminal.
pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    rx: Receiver<Vec<u8>>,
    alive: Arc<AtomicBool>,
    killer: Box<dyn ChildKiller + Send + Sync>,
}

impl PtySession {
    /// Open a PTY, launch `shell` with its working directory set to `cwd`, and start
    /// the reader thread. `on_output` is invoked whenever new bytes arrive (and once
    /// more when the shell exits) — the UI passes a cloned `egui::Context` repaint.
    pub fn spawn<F>(shell: &str, cwd: &Path, rows: u16, cols: u16, on_output: F) -> Result<Self>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(shell);
        cmd.cwd(cwd);
        let mut child = pair.slave.spawn_command(cmd)?;
        let killer = child.clone_killer();

        // Drop the slave so the reader observes EOF when the child exits (required on
        // Windows ConPTY, harmless on unix).
        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let alive = Arc::new(AtomicBool::new(true));
        let on_output = Arc::new(on_output);

        // Reader thread: stream output to the UI. Breaks on EOF/error.
        {
            let on_output = Arc::clone(&on_output);
            thread::spawn(move || {
                let mut buf = [0u8; 4096];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            if tx.send(buf[..n].to_vec()).is_err() {
                                break; // UI dropped the receiver
                            }
                            on_output();
                        }
                    }
                }
            });
        }

        // Waiter thread: detect exit reliably via `child.wait()` rather than relying
        // on the reader observing EOF — ConPTY can leave the read blocked after the
        // shell has already exited.
        {
            let alive = Arc::clone(&alive);
            let on_output = Arc::clone(&on_output);
            thread::spawn(move || {
                let _ = child.wait();
                alive.store(false, Ordering::SeqCst);
                on_output(); // wake the UI so it can observe the exit
            });
        }

        Ok(Self {
            master: pair.master,
            writer,
            rx,
            alive,
            killer,
        })
    }

    /// Write raw bytes (keystrokes, command lines) to the shell's stdin.
    pub fn write(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Pull the next available output chunk, if any. Non-blocking.
    pub fn try_recv(&self) -> Option<Vec<u8>> {
        self.rx.try_recv().ok()
    }

    /// Tell the PTY the visible grid was resized.
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    /// Whether the shell process is still running.
    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        // Best-effort: kill the shell when the session is dropped (e.g. app close).
        let _ = self.killer.kill();
    }
}
