# Architectural Decisions

## 2026-05-21 — Native-only desktop app, no WASM/browser runtime (sprint 0)
- **Context:** `RUSTY_PROMPT.md` v2 superseded v1's browser-first WASM design (which had three competing executor strategies).
- **Decision:** Rusty is a native `eframe` desktop app for macOS/Linux/Windows. No WASM build of Rusty itself; no hosted website. Distribution is `git clone` + `install.sh`/`install.ps1` + `cargo build --release`.
- **Alternatives considered:** v1's WASM/`trunk` browser deployment — rejected by the v2 spec rewrite (rust-in-browser was research-tier and broke the bundle/backend goals).
- **Consequences:** Free to depend on the system Rust toolchain at runtime and shell out to real subprocesses. The `rusty-app` crate strips all WASM scaffolding from `eframe_template`.

## 2026-05-21 — Three-process executor; the compiler is on disk (sprint 0)
- **Context:** Exercises must run real Rust, not a simulation, per the "real-tool learning" pedagogy (§1.5).
- **Decision:** The entire executor is three subprocesses owned by `rusty-host`: (1) a PTY-attached sandbox shell (`portable-pty`), (2) `cargo test --message-format=json` grading parsed via `cargo_metadata`, (3) a `rust-analyzer` LSP subprocess (`lsp-types`). No `SynPatternExecutor`, no hosted endpoint, no rust-in-WASM.
- **Alternatives considered:** v1's swappable `Executor` trait (syn pattern-match / playground / cranelift-wasm) — obsoleted by going native, where the real compiler is simply present.
- **Consequences:** Phase 1 PTY spike is the single highest-risk task. ANSI rendering: try `egui_term` first, fall back to `vte` + a custom egui widget (decided in the Phase-1 spike; `egui_term` is only at 0.1.0 and likely egui-version-stale).

## 2026-05-21 — Crate-portability boundary (sprint 0)
- **Context:** Prompt §11 tripwire: OS/subprocess code must not leak into the engine crates.
- **Decision:** `rusty-curriculum`, `rusty-scheduler`, `rusty-grader`, `rusty-pedagogy` stay OS-portable (no `std::process`, no raw FS, no PTY). All OS-dependent code lives in `rusty-host` and `rusty-terminal`. `rusty-grader` grades already-captured cargo output passed in by the host; it never spawns cargo itself.
- **Alternatives considered:** Single-crate app, split later — rejected; the boundary is far cheaper to enforce as a compile-time crate split from the skeleton than to retrofit.
- **Consequences:** A `cargo-deny`-style portability guard (`T-PORT-GUARD`, backlog) lands in Phase 1 once real code exists.

## 2026-05-21 — Cargo virtual workspace with glob members + shared dependency table (sprint 0)
- **Context:** Seven crates; the per-task commit gate requires every committed state to compile green, and a virtual workspace is invalid with zero members.
- **Decision:** Root is a virtual manifest (`resolver = "2"`, `members = ["crates/*"]`, `[workspace.dependencies]` pinning real versions). The root + the first members (the four portable crates) ship in one commit; later crate tasks auto-join via the glob without re-editing the root manifest. Pins resolved from crates.io 2026-05-21: eframe/egui 0.34, serde 1, toml 1, anyhow 1, thiserror 2.
- **Alternatives considered:** Explicit `members` list (forces a root-manifest edit per crate, creating Touches overlap between tasks) — rejected for the glob.
- **Consequences:** New crates are added by creating a dir under `crates/`; no root-manifest churn.

## 2026-05-21 — Toolchain pinned to `channel = "stable"`, not an exact version (sprint 0)
- **Context:** Prompt §0 says "MSRV: latest stable at install time." Rusty is distributed as clone + `cargo build` (the bootstrap *is* the first lesson).
- **Decision:** `rust-toolchain.toml` pins `channel = "stable"` with `rustfmt` + `clippy` components and **no** `targets` entry (each platform builds for its own host). Exact version at authoring time (1.93.1) is documented in an in-file comment.
- **Alternatives considered:** Pin exact `1.93.1` + a Windows target (the original plan Note) — rejected: it would force learners to download a stale toolchain and a wrong cross-target.
- **Consequences:** Reproducible-build stretch goal is relaxed in favor of learners always using their latest stable; pin to a release train later if reproducibility becomes a hard requirement.

## 2026-05-21 — eframe 0.34 `App::ui` is the implementation surface (sprint 0)
- **Context:** eframe 0.34.2 changed the `App` trait.
- **Decision:** Implement `fn ui(&mut self, ui: &mut egui::Ui, _frame)` (the required method; the framework wraps the central panel) rather than the pre-0.34 `fn update` + manual `CentralPanel::show` (now deprecated).
- **Consequences:** UI code receives a central `Ui` directly; panels/splits for the two-pane layout (§2) are added inside `ui` in later phases.

## 2026-05-21 — Solo trunk-based development on `main` (sprint 0)
- **Context:** Greenfield solo repo; the user engaged auto mode and stepped away.
- **Decision:** Per-task commits straight to `main` (no per-sprint feature branch / PR), pushed to `origin/main`. The sprint structure (research/plan/test reports, decisions log, per-task commit boundaries) provides the review surface.
- **Alternatives considered:** PR-per-sprint with auto-merge — viable but adds ceremony with no second reviewer; revisit if a collaborator joins.
- **Consequences:** `main` is always green (every commit passed the gate); history reads as one commit per task.
