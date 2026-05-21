# Completed Tasks Log (Append-Only)

## T-001 (sprint 0)
- **Description:** Establish the Cargo virtual workspace (root `Cargo.toml`, resolver 2, `members = ["crates/*"]`, `[workspace.dependencies]` pinned) with the four portable engine crates as initial members.
- **Completed:** 2026-05-21T06:35:14Z
- **Files modified:** `Cargo.toml`, `crates/rusty-curriculum/{Cargo.toml,src/lib.rs}`, `crates/rusty-scheduler/{Cargo.toml,src/lib.rs}`, `crates/rusty-grader/{Cargo.toml,src/lib.rs}`, `crates/rusty-pedagogy/{Cargo.toml,src/lib.rs}`
- **Commit:** `fd44ddd`

## T-002 (sprint 0)
- **Description:** Pin the toolchain in `rust-toolchain.toml` with rustfmt + clippy components. Deviation from plan Notes: pinned `channel = "stable"` (not exact `1.93.1`) and omitted a `targets` pin — correct for a cross-platform clone+build teaching app; locked EARS criteria (reports stable channel; declares rustfmt/clippy) are met. Exact version documented in-file.
- **Completed:** 2026-05-21T06:36:30Z
- **Files modified:** `rust-toolchain.toml`
- **Commit:** `7a57160`

## T-003 (sprint 0)
- **Description:** Add the two host-boundary lib crates `rusty-host` and `rusty-terminal` as workspace members (placeholder libs; designated home for all PTY/subprocess/OS code). Workspace now has 6 members.
- **Completed:** 2026-05-21T06:38:00Z
- **Files modified:** `crates/rusty-host/{Cargo.toml,src/lib.rs}`, `crates/rusty-terminal/{Cargo.toml,src/lib.rs}`
- **Commit:** `47f5a93`

## T-004 (sprint 0)
- **Description:** Add the `rusty-app` eframe binary (`[[bin]]` name `rusty`) with `voice.rs` (centralized `WINDOW_TITLE`) and `main.rs` running `eframe::run_native`. Adapted to eframe 0.34's new `App` trait: the required method is `fn ui(&mut self, ui, frame)` (framework wraps the central panel), replacing the old `update` + manual `CentralPanel::show` (now deprecated). Workspace now has 7 members. Window-launch is a manual Test-Phase smoke check.
- **Completed:** 2026-05-21T06:43:00Z
- **Files modified:** `crates/rusty-app/{Cargo.toml,src/main.rs,src/voice.rs}`
- **Commit:** `f47a30b`
