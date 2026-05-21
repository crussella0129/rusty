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

## T-005 (sprint 0)
- **Description:** Add a marker-commented `rusty runtime` block to `.gitignore` excluding `workspace/` (per-lesson learner sandbox) and `.rusty-state/` (progress store). Verified: synthetic files under both paths are not listed by `git status --porcelain`.
- **Completed:** 2026-05-21T06:45:00Z
- **Files modified:** `.gitignore`
- **Commit:** `ac795a9`

## T-006 (sprint 0)
- **Description:** Scaffold the directory tree: `content/lessons/.gitkeep`, `assets/.gitkeep`, and four `docs/` stubs (ARCHITECTURE, PEDAGOGY, CONTENT_AUTHORING, INSTALL) each with YAML frontmatter (`project`/`tags`/`related`/`status: stub`), `[[wikilinks]]`, and a "filled in Phase N" pointer. `workspace/` intentionally not tracked (gitignored).
- **Completed:** 2026-05-21T06:47:00Z
- **Files modified:** `content/lessons/.gitkeep`, `assets/.gitkeep`, `docs/ARCHITECTURE.md`, `docs/PEDAGOGY.md`, `docs/CONTENT_AUTHORING.md`, `docs/INSTALL.md`
- **Commit:** `5e41b1b`

## T-101 (sprint 1)
- **Description:** Add `portable-pty = "0.9"` + `vte = "0.15"` to `[workspace.dependencies]`; wire `portable-pty` + `anyhow` into `rusty-host`, `vte` + `egui` into `rusty-terminal`. Workspace compiles clean.
- **Completed:** 2026-05-21T14:31:13Z
- **Files modified:** `Cargo.toml`, `crates/rusty-host/Cargo.toml`, `crates/rusty-terminal/Cargo.toml`
- **Commit:** `ebeb2d8`

## T-102 (sprint 1)
- **Description:** `rusty-host::sandbox` — `default_shell()` (pwsh/cmd on Windows, $SHELL//bin/bash on unix) and pure `resolve_cd(line, cwd, root) -> CdOutcome{Allowed|Refused|NotCd}` using lexical segment-stack normalization (handles `..`, absolute, drive-root, nested `a/../b`). 10 unit tests pass.
- **Completed:** 2026-05-21T14:34:00Z
- **Files modified:** `crates/rusty-host/src/sandbox.rs`, `crates/rusty-host/src/lib.rs`
- **Commit:** `82a3e52`

## T-103 (sprint 1)
- **Description:** `rusty-host::pty::PtySession` — opens a PTY via `portable-pty`, spawns the shell with `cwd=sandbox`, drops the slave, runs a reader thread (→ mpsc + repaint callback) and a separate `child.wait()` waiter thread for reliable exit detection; exposes `write`/`try_recv`/`resize`/`is_alive`, kills the child on Drop. 3 integration tests pass (echo round-trip, resize, exit→not-alive). **Build discovery:** Windows ConPTY withholds ALL output until the terminal answers its startup `ESC[6n` cursor-position query with a CPR report — the integration tests include a minimal DSR responder; the real renderer/app must answer `ESC[6n` with the grid's actual cursor pos (handled in T-104/T-106). Also: exit detection via reader-EOF is unreliable on ConPTY, hence the dedicated waiter thread.
- **Completed:** 2026-05-21T14:42:00Z
- **Files modified:** `crates/rusty-host/src/pty.rs`, `crates/rusty-host/src/lib.rs`, `crates/rusty-host/tests/pty_roundtrip.rs`
- **Commit:** `59847a9`

## T-104 (sprint 1)
- **Description:** `rusty-terminal` grid model + `vte` performer. `Cell{ch,fg,bg,bold}` + 16-color ANSI palette (`cell.rs`); `Grid` holding cells, cursor, the current SGR pen, and a CPR reply queue, with put/wrap/scroll/line-feed/CR/tab/backspace/cursor-move/erase/SGR + `queue_cpr()`/`take_replies()` (`grid.rs`); `Performer` impl of `vte::Perform` translating print/execute/csi (SGR `m`, cursor `H/f/A-D`, erase `J/K`, DSR `n`→CPR) (`performer.rs`). 8 unit tests pass (color+reset, bold, CRLF, erase, wrap, DSR→`ESC[1;1R`, absolute cursor). DSR/CPR handling resolves the T-103 ConPTY discovery.
- **Completed:** 2026-05-21T14:52:00Z
- **Files modified:** `crates/rusty-terminal/src/{cell.rs,grid.rs,performer.rs,lib.rs}`
- **Commit:** `841a32f`
