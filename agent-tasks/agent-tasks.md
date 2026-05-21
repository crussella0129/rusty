# Agent Tasks (Persistent Backlog)

- [ ] T-105 (sprint 1): rusty-terminal::widget terminal_ui (paint + input) — touches: `crates/rusty-terminal/src/{widget.rs,lib.rs}`
- [ ] T-106 (sprint 1): wire PTY+grid+two-pane layout into rusty-app — touches: `crates/rusty-app/src/{main.rs,voice.rs}`

- [ ] T-PORT-GUARD (deferred → Phase 1 / sprint 1): add an automated portability guard (e.g. `cargo-deny` rule or `#![forbid]`-style contract) preventing `std::process`/OS-FS/PTY deps from leaking into the four portable crates (rusty-curriculum, rusty-scheduler, rusty-grader, rusty-pedagogy). Cheap to add once real code exists. — touches: `deny.toml` or crate-level lints. (From s0 plan critique C-006.)
