# Agent Tasks (Persistent Backlog)

- [ ] T-003 (sprint 0): Add host-boundary crates rusty-host + rusty-terminal — touches: `crates/rusty-{host,terminal}/{Cargo.toml,src/lib.rs}`
- [ ] T-004 (sprint 0): Add rusty-app eframe binary + voice.rs — touches: `crates/rusty-app/{Cargo.toml,src/main.rs,src/voice.rs}`
- [ ] T-005 (sprint 0): Exclude workspace/ and .rusty-state/ from git — touches: `.gitignore`
- [ ] T-006 (sprint 0): Scaffold directory tree + docs stubs — touches: `content/lessons/.gitkeep`, `assets/.gitkeep`, `docs/*.md`

- [ ] T-PORT-GUARD (deferred → Phase 1 / sprint 1): add an automated portability guard (e.g. `cargo-deny` rule or `#![forbid]`-style contract) preventing `std::process`/OS-FS/PTY deps from leaking into the four portable crates (rusty-curriculum, rusty-scheduler, rusty-grader, rusty-pedagogy). Cheap to add once real code exists. — touches: `deny.toml` or crate-level lints. (From s0 plan critique C-006.)
