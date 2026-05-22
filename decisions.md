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

## 2026-05-21 — Roll our own vte-based terminal renderer, not egui_term (sprint 1)
- **Context:** Phase 1 needs an embedded ANSI terminal on egui 0.34. The ANSI-renderer choice was deferred from Phase 0.
- **Decision:** Build our own renderer on `vte` 0.15 (parse) + a `Grid`/`Cell` model + an egui widget. Reject `egui_term` and `egui-terminal`.
- **Alternatives considered:** `egui_term` 0.1.0 (published) pins egui 0.31; `egui-terminal` 0.1.0 pins egui 0.28 — both incompatible with our 0.34. `egui_term`'s 0.34 support exists only on an unpublished git branch and drags the heavy `alacritty_terminal` backend. A git dependency on a moving rev was rejected.
- **Consequences:** Full control over the `vte`→grid→egui path; renderer is pure and unit-testable. Scrollback / 256-color / alt-screen / mouse are deferred. If maintenance cost grows, revisit a published `egui_term`.

## 2026-05-21 — Windows ConPTY requires answering the DSR cursor-position query (sprint 1)
- **Context:** On Windows, the embedded shell produced no output and never exited in early testing.
- **Decision:** The terminal must answer ConPTY's startup `ESC[6n` (Device Status Report) with a CPR report `ESC[row;colR` computed from the grid cursor; the app forwards `Grid::take_replies()` to the PTY each frame. Exit detection uses a dedicated `child.wait()` thread, not reader-EOF (unreliable on ConPTY).
- **Alternatives considered:** Ignoring DSR (shell hangs forever); relying on reader-EOF for exit (never fires reliably on ConPTY).
- **Consequences:** pwsh/cmd both work in the embedded terminal. Any future DSR/DA queries the shell makes must likewise be answered. Headless PTY tests use `cmd.exe` + a fixed DSR responder to isolate plumbing from PSReadLine.

## 2026-05-21 — egui 0.34 multi-panel layout via `show_inside` on `App::ui` (sprint 1)
- **Context:** The two-pane layout (lesson | terminal) needs side + central panels. The s0 ADR chose `App::ui`.
- **Decision:** Keep `App::ui` (the required eframe-0.34 method) and nest panels with `Panel::left(..).show_inside(ui, ..)` + `CentralPanel::default().show_inside(ui, ..)`. The `Context`-level `SidePanel::show`/`CentralPanel::show`/`default_width` are deprecated in 0.34.
- **Alternatives considered:** Switching to `App::update(ctx, ..)` with `Context`-level panels (the s1 build-plan Note suggested this) — unnecessary and uses deprecated APIs. Rejected; the plan Note was wrong.
- **Consequences:** s0 `App::ui` ADR stands (no supersession). All layout uses `show_inside`.

## 2026-05-21 — Own a pulldown-cmark prose renderer, not egui_commonmark (sprint 2)
- **Context:** Lesson prose is Markdown; it must render in egui 0.34.
- **Decision:** Render with a small in-house `pulldown-cmark`→egui renderer (`rusty-app::markdown`): a pure `to_blocks(&str) -> Vec<MdBlock>` (headings, paragraphs, inline code, bold/italic, fenced code, bullets) plus an egui draw. Reject `egui_commonmark`.
- **Alternatives considered:** `egui_commonmark` 0.23 — same egui-version coupling risk that made `egui_term` unusable, heavier deps (egui_extras/image). A prose subset is all lessons need.
- **Consequences:** Full control, version-risk-free, unit-testable parse step. Tables/images/nested-lists degrade to text; revisit if richer prose is needed.

## 2026-05-21 — Curriculum model: internally-tagged enums + pure parse; lesson projects detach (sprint 2)
- **Context:** Lessons are `lesson.toml` files; the parser must stay OS-portable; lesson cargo projects live under the repo workspace.
- **Decision:** `rusty-curriculum` is pure — `parse_lesson(&str)`; the FS read + starter→sandbox copy live in `rusty-host`. `Block`/`RecallPrompt`/`Exercise` use serde **internally-tagged** enums (`kind="..."`) — confirmed working with `toml` 1.x via a Phase-2 spike (no adjacently-tagged fallback needed). Each lesson `starter`/`solution` `Cargo.toml` carries an empty `[workspace]` table so the learner's `cargo run` resolves inside the copied sandbox; the root workspace `exclude`s `content`/`workspace`. `prepare_sandbox` skips `target/` and reads-before-creating (no corrupt empty sandbox).
- **Consequences:** The model is trivially testable and portable; new lessons are pure data + a detached cargo project. Sandbox copies are clean (no build artifacts).

## 2026-05-21 — Grading signal = cargo exit status + error diagnostics, not libtest JSON (sprint 3)
- **Context:** The grader needs to distinguish compile error / test failure / pass from a `cargo test` run.
- **Decision:** A diagnostic with `level == Error` ⇒ `CompileError`; otherwise cargo's exit status decides `Pass`/`TestsFailed`. Per-test (libtest) JSON breakdown is NOT used.
- **Alternatives considered:** libtest `--format=json` for per-test results — unstable on stable Rust, violates the pinned-stable-toolchain ADR. Rejected.
- **Consequences:** `TestsFailed` is coarse (some test failed, not which). Sufficient for the spike; revisit if per-test naming is needed.

## 2026-05-21 — Grader owns a Diag model decoupled from cargo_metadata; sandboxes build isolated (sprint 3)
- **Context:** `rusty-grader` parses `cargo --message-format=json` via `cargo_metadata`, but the UI shouldn't depend on that crate's types.
- **Decision:** A local `Diag{code,level,message,rendered,primary_span}`/`Level`/`Span` is the public surface; `parse_diagnostics` maps `cargo_metadata`'s `Diagnostic` into it (keeping the verbatim `rendered` text). `rusty-grader` stays portable (parses captured bytes; `cargo_metadata` is a pure parser). `rusty-host::grade` spawns cargo (process #2) with `.env_remove("CARGO_TARGET_DIR")` so each sandbox builds in its own `target/` (isolation even when an outer `CARGO_TARGET_DIR` is set).
- **Consequences:** Sprint-4 UI consumes `Diag`/`Verdict`/`concept_for_code` without a `cargo_metadata` dep. Grading is a separate subprocess from the PTY.

## 2026-05-22 — GUI grading runs on a background thread; pure `Annotation` model in the grader (sprint 4)
- **Context:** A learner presses Check and `cargo test` can take seconds; running it in `App::ui` would freeze the egui event loop. Also, the result needs an on-screen rustc-style pane without coupling `rusty-app` to `cargo_metadata`.
- **Decision:** `rusty-app::start_grade` spawns `rusty_host::grade` (process #2) on a `std::thread`, delivers a `Result<Verdict,String>` over an `mpsc` channel, and `poll_grade` drains it each frame with `ctx.request_repaint()` — the same thread+repaint pattern the PTY uses. The render model is a pure `annotate(&Verdict) -> Annotation{headline, body_blocks(verbatim rustc text), links}` living in the portable `rusty-grader`; `rusty-app` draws it (no `cargo_metadata`). The channel-delivered mapping is factored into a pure `grade_outcome` for testing; the thread-spawn + repaint glue is verified via the visual heartbeat.
- **Alternatives considered:** Synchronous grade in `App::ui` (freezes the window on every Check) — rejected. Putting the annotation draw model in `rusty-app` — rejected (would either duplicate the verdict types or pull `cargo_metadata` into the app).
- **Consequences:** The UI stays responsive ("checking…" while in flight). One grade at a time (`grade_job.is_some()` guard). A future async/cancel story can replace the channel without touching the pure model.

## 2026-05-22 — Editor syntax highlighting via egui_extras' built-in fallback, not syntect (sprint 4)
- **Context:** The code editor needs Rust syntax highlighting on egui 0.34 (prompt §6 sanctions `egui_extras::syntax_highlighting` *or* syntect).
- **Decision:** Use `egui_extras::syntax_highlighting::{highlight, CodeTheme}` as a `TextEdit::multiline` layouter with **no extra feature** — the module ships in the default build and its built-in fallback highlights Rust adequately; the heavy `syntect` feature is deliberately not enabled. Galley built via `ui.painter().layout_job` (the `&self` path; `Fonts::layout_job` needs `&mut`).
- **Alternatives considered:** Enabling `syntect` (accurate but a heavy dep) — rejected, same light-deps stance that rejected `egui_term`/`egui_commonmark`. (Discovery: egui_extras has no `syntax_highlighting` feature; only `syntect` is gated.)
- **Consequences:** Zero new heavy deps; highlighting is "okish" for Rust (good enough for a learner editor). Revisit `syntect` only if multi-language accuracy becomes a need.

## 2026-05-22 — Sandboxed editor file I/O with an explicit `target/` denial; lesson-1 Faded surfaces E0425 on purpose (sprint 4)
- **Context:** The editor reads/writes sandbox files; a crafted relative path must not escape, and the build-artifact `target/` must be off-limits. Separately, the live concept-link UI path needs to be reachable now (only lesson 1 is authored).
- **Decision:** `rusty-host::files` routes every read/write through `sandbox::contain(root, rel)` (reuses the lexical `resolve_cd` normalization, rebuilt onto the real root) **plus a separate explicit first-segment `target/` denial** — `contain` alone would allow `target/` since it lives *inside* the sandbox. Lesson 1's Faded exercise is authored so the *unfilled* starter calls an undefined `greeting()` from a `#[cfg(test)]` test → unedited `cargo test` yields **E0425**, which `concept_for_code` maps to the authored `foundations-01-hello` → a genuinely live concept-link during the heartbeat; the `#[cfg(test)]` placement keeps `cargo run` compiling regardless.
- **Alternatives considered:** Folding `target/` into the containment check (it can't — `target/` is contained); using `todo!()` in the Faded blank (compiles → runtime panic → `TestsFailed`, no E0425, no live link) — rejected.
- **Consequences:** Editing is safe-by-default; the live-link render path is exercised without authoring lessons 02–04. When later lessons exist, their error codes light up automatically (the map is already forward-looking).
