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
