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

## T-105 (sprint 1)
- **Description:** `rusty-terminal::widget` — `terminal_ui(ui, &grid, writer)` paints the grid (per-cell bg rects + per-row colored `LayoutJob` galleys + cursor block) and, when focused, forwards `Event::Text` and mapped keys to the writer. Pure helpers `key_to_bytes(key, mods)` (Enter→`\r`, Backspace→`0x7f`, arrows→`\x1b[A-D`, Ctrl-C→`0x03`, Ctrl-D→`0x04`, printables→None) and `grid_dims(avail, char_w, row_h)`. 6 widget unit tests pass (14 total in crate). **API note:** egui 0.34 `FontsView` metrics need `&mut` → used `ui.fonts_mut`; galley built via `painter.layout_job`.
- **Completed:** 2026-05-21T15:00:00Z
- **Files modified:** `crates/rusty-terminal/src/{widget.rs,lib.rs}`
- **Commit:** `8887089`

## T-106 (sprint 1)
- **Description:** Wire the PTY + terminal into `rusty-app`. `RustyApp` holds a `rusty_terminal::Terminal` (parser+grid wrapper, added to keep `rusty-app` off a direct `vte` dep), a `PtySession` spawned in `workspace/lessons/spike/` with a cloned `egui::Context` repaint callback, and a mirrored input line for `cd` interception. `App::ui` drains output→grid, answers DSR/CPR replies, lays out `Panel::left` (lesson placeholder) + `CentralPanel` terminal via `show_inside`, resizes grid+PTY to the fitted dims, and forwards keystrokes — refusing sandbox-escaping `cd` (Ctrl-C + `voice::CD_REFUSED`). New voice copy. Full workspace: 33 tests pass; clippy/fmt clean; 7s GUI smoke launch OK (window + PTY alive). **API note:** eframe 0.34 keeps `App::ui` required (no ADR supersession after all); panels use `show_inside` and `Panel::left`/`default_size` (the `Context`-level `.show()`/`SidePanel`/`default_width` are deprecated).
- **Completed:** 2026-05-21T15:12:00Z
- **Files modified:** `crates/rusty-app/src/{main.rs,voice.rs}`, `crates/rusty-app/Cargo.toml`, `crates/rusty-terminal/src/{terminal.rs,lib.rs}`
- **Commit:** `4f9e16c`

## T-201 (sprint 2)
- **Description:** Typed curriculum model in `rusty-curriculum` (`Lesson`, `Track`, `Concept`, `LessonId`/`ConceptId`, internally-tagged `Block`/`RecallPrompt`/`Exercise`/`SuccessCriterion`, `Reference`, `CalloutTone`) with serde. **C-001 spike PASSED:** `toml` 1.x deserializes internally-tagged enums from `[[body]]`/`[recall_prompt]` tables — no adjacently-tagged fallback needed. Added `serde`/`toml`/`thiserror` deps; added root `Cargo.toml` `exclude=["content","workspace"]` (C-003). 2 deser tests pass; crate stays portable.
- **Completed:** 2026-05-21T18:48:31Z
- **Files modified:** `crates/rusty-curriculum/{Cargo.toml,src/lib.rs,src/model.rs}`, `Cargo.toml`
- **Commit:** `acc6277`

## T-202 (sprint 2)
- **Description:** Pure `parse_lesson(&str) -> Result<Lesson, CurriculumError>` (`toml::from_str` + validation: non-empty id/title/body) with `thiserror` `CurriculumError{Parse,Invalid}`. 3 tests (valid parse, invalid TOML→Parse, empty id→Invalid). No filesystem (portability).
- **Completed:** 2026-05-21T18:49:24Z
- **Files modified:** `crates/rusty-curriculum/src/{loader.rs,lib.rs}`
- **Commit:** `13460e5`

## T-203 (sprint 2)
- **Description:** `rusty-host::content` — `load_lesson(dir)` (read lesson.toml + delegate to pure `parse_lesson`) and `prepare_sandbox(content_dir, workspace_root, id)` (recursive `starter/`→`workspace/lessons/<id>/` copy, idempotent so learner edits survive). Added `rusty-curriculum` path dep. 3 integration tests with temp fixtures (copies starter, idempotent-preserves-edit, load from temp). Real-lesson load test added with T-204 content.
- **Completed:** 2026-05-21T18:50:53Z
- **Files modified:** `crates/rusty-host/src/{content.rs,lib.rs}`, `crates/rusty-host/Cargo.toml`, `crates/rusty-host/tests/content.rs`
- **Commit:** `a8cd61f`

## T-204 (sprint 2)
- **Description:** Authored lesson 1 `foundations-01-hello`: `lesson.toml` (Foundations, 3 concepts, prose+code+now_run+tip-callout body, MC recall, a further-reading ref), `starter/` + `solution/` cargo projects each with an empty `[workspace]` table (detaches from Rusty's workspace so the learner's `cargo run` resolves). Verified: `load_lesson` real-content test passes; `cargo run` in starter prints the greeting; `cargo check --workspace` cleanly excludes the nested project; starter `target/` is gitignored. Exercises empty (Phase 3 — T-EX1 backlog).
- **Completed:** 2026-05-21T18:53:12Z
- **Files modified:** `content/lessons/foundations-01-hello/{lesson.toml,starter/*,solution/*}`, `crates/rusty-host/tests/content.rs`
- **Commit:** `3483ed7`

## T-205 + T-206 (sprint 2)
- **Description:** Lesson renderer + app integration, landed as one green commit (the egui render fns have no caller until wired, so splitting would cascade dead-code warnings through `voice` consts — combined per "coherent diff"). **T-205:** owned `markdown` module (`pulldown-cmark`→egui; pure unit-tested `to_blocks` for headings/paragraphs/bold/italic/inline-code/fenced-code/bullets + `render_markdown`) and `lesson_view::render` (prose via markdown, code monospaced in a frame, NowRun as `voice::RUN_PROMPT_PREFIX`+command, callouts boxed). Added `pulldown-cmark` to workspace deps + `rusty-app`. **T-206:** `RustyApp` gains `lesson: Option<Lesson>` + `load_error`; infallible `new()` does load_lesson→prepare_sandbox→spawn PTY in the lesson sandbox (fallback on error); left `Panel` renders lesson 1 via `lesson_view` (or an error label); removed `LESSON_PANE_PLACEHOLDER`. 49 workspace tests pass; clippy/fmt clean; 7s GUI smoke OK.
- **Completed:** 2026-05-21T18:58:38Z
- **Files modified:** `Cargo.toml`, `crates/rusty-app/{Cargo.toml,src/markdown.rs,src/lesson_view.rs,src/main.rs,src/voice.rs}`
- **Commit:** `c9399b7`

## T-301 (sprint 3)
- **Description:** `rusty-grader` diagnostic model + parser + verdict. Local `Diag{code,level,message,rendered,primary_span}`/`Level`/`Span` (decoupled from cargo_metadata); `parse_diagnostics(json)` via `Message::parse_stream(...).filter_map(Result::ok)` keeping `CompilerMessage` (code via `.map(|c| c.code)`; non_exhaustive `DiagnosticLevel`→`Level::Other` wildcard; primary span = `is_primary`); `Verdict{Pass,CompileError,TestsFailed,RunMismatch}` + `grade_cargo_test`/`verdict_from_diags`/`grade_run_output` (CRLF→LF + trailing-ws/blank-line normalization). Added cargo_metadata+serde. 8 unit tests pass incl. a **real captured E0382 fixture** (validates the cargo_metadata API). Crate stays portable.
- **Completed:** 2026-05-21T23:34:21Z
- **Files modified:** `crates/rusty-grader/{Cargo.toml,src/lib.rs,src/diagnostic.rs,src/verdict.rs,tests/fixtures/{e0382.json,build_finished.json}}`, `Cargo.toml`
- **Commit:** `effb2f7`

## T-302 (sprint 3)
- **Description:** `concept_for_code(&str) -> Option<&'static str>` mapping Foundations rustc error codes to lesson ids (E0382→ownership-moves; E0499/E0502/E0505/E0506/E0106/E0621→borrows; E0308→variables; E0425/E0433→hello). Forward-looking (returns ids for not-yet-authored lessons; renderer links only existing ones). 3 unit tests.
- **Completed:** 2026-05-21T23:36:00Z
- **Files modified:** `crates/rusty-grader/src/{error_map.rs,lib.rs}`
- **Commit:** `6ad3e8a`

## T-303 (sprint 3)
- **Description:** `evaluate(&SuccessCriterion, &CargoOutcome) -> Verdict` bridging the curriculum criterion to the grader; `CargoOutcome{test_json,test_exit_ok,run_stdout}`. Added `rusty-curriculum` dep (portable). 2 unit tests (CargoTestPasses→Pass; CargoRunOutputMatches match→Pass / mismatch→RunMismatch).
- **Completed:** 2026-05-21T23:37:32Z
- **Files modified:** `crates/rusty-grader/{Cargo.toml,src/lib.rs,src/evaluate.rs}`
- **Commit:** `2bb94fe`

## T-304 (sprint 3)
- **Description:** `rusty-host::grade` — `run_cargo_test(sandbox) -> (json, exit_ok)` and `run_cargo_run(sandbox) -> stdout` via `std::process::Command` (current_dir=sandbox, stdout captured), and `grade(sandbox, &SuccessCriterion) -> Verdict` running only the needed cargo command + calling the grader. Added `rusty-grader` dep; re-exported `Verdict`. Process #2 of the three-process model (separate Command, not the PTY). 4 integration tests with real temp cargo projects (passing test→Pass; borrow error→CompileError w/ E0382; failing test→TestsFailed; run output match→Pass / mismatch→RunMismatch). Full workspace: 69 tests pass.
- **Completed:** 2026-05-21T23:41:54Z
- **Files modified:** `crates/rusty-host/{Cargo.toml,src/grade.rs,src/lib.rs,tests/grade.rs}`
- **Commit:** `bc3e8b9`

## T-401 (sprint 4)
- **Description:** Sandboxed editor file I/O in `rusty-host::files` — `list_sandbox_rs_files` (relative `.rs` paths, sorted, skips `target/`), `read_sandbox_file`, `write_sandbox_file` (creates parent dirs in-sandbox). All routed through a new `sandbox::contain(root, rel) -> Option<PathBuf>` guard (reuses the existing lexical normalize/starts_with logic, rebuilds onto the real root for valid fs paths) plus a *separate* explicit `target/` first-segment denial (C-001: `target/` is inside the sandbox, so containment alone would allow it). All editor `std::fs` stays in this crate (§11). 5 integration tests (list-skips-target, read/write round-trip + parent-dir creation, `../` escape refused, absolute-path refused, `target/` write refused).
- **Completed:** 2026-05-22T06:20:00Z
- **Files modified:** `crates/rusty-host/src/{files.rs,sandbox.rs,lib.rs}`, `crates/rusty-host/tests/files.rs`
- **Commit:** `6562cd4`

## T-402 (sprint 4)
- **Description:** Pure annotation model in `rusty-grader::annotate` — `annotate(&Verdict) -> Annotation{headline: Headline, body_blocks: Vec<String>, links: Vec<ConceptLink>}`. Body blocks are each error's verbatim rustc `rendered` text (fallback: message); links come from `concept_for_code`, deduplicated. Keeps `rusty-app` free of `cargo_metadata` (UI consumes plain owned types). Forward-looking: emits a link for any known code; the UI gates navigability. 7 unit tests incl. the C-002 case (E0425 → `foundations-01-hello`, the authored lesson) and link-dedup. Crate stays portable.
- **Completed:** 2026-05-22T06:30:00Z
- **Files modified:** `crates/rusty-grader/src/{annotate.rs,lib.rs}`
- **Commit:** `b75c156`

## T-403 (sprint 4)
- **Description:** egui code editor in `rusty-app::editor` — `Editor` over a lesson sandbox: a file picker (`selectable_label` per `.rs` file from `list_sandbox_rs_files`, prefers `main.rs`/`lib.rs`), a `TextEdit::multiline().code_editor()` whose `layouter` calls `egui_extras::syntax_highlighting::highlight` (built-in fallback, no `syntect`) and builds the galley via `ui.painter().layout_job` (the `&self` path; `Fonts::layout_job` needs `&mut`), and a Save that writes through the host guard. Pure `load_contents` split out for testing. Wired into `main.rs`: layout is now lesson (left) | editor (centre) | terminal (right `Panel::right`). **Build notes:** egui_extras has no `syntax_highlighting` feature (the module ships by default; only `syntect` is gated) — added `egui_extras.workspace = true` with no extra feature; layouter closure sig in egui 0.34 is `FnMut(&Ui, &dyn TextBuffer, f32) -> Arc<Galley>`. 3 unit tests (pure load clears dirty; headless render of a loaded buffer exercises the highlight layouter; headless render of an empty sandbox shows the no-files notice). 12 app tests pass.
- **Completed:** 2026-05-22T06:55:00Z
- **Files modified:** `crates/rusty-app/src/{editor.rs,main.rs,voice.rs}`, `crates/rusty-app/Cargo.toml`
- **Commit:** `d225b7a`

## T-404 (sprint 4)
- **Description:** Annotation pane + exercise UI + threaded grading. `annotation.rs` draws a `rusty_grader::Annotation` (headline+colour, verbatim rustc body blocks monospaced, concept links — a live `ui.link` when `link_is_available` against the loaded lesson ids, else a weak "coming soon" label). `exercise_view.rs` renders all four `Exercise` variants and returns the `SuccessCriterion` to grade on a Check press; `criterion_for_exercise` (Some for Faded/Open, None for Worked/PredictThenRun) is the single gate for the Check control; `ExerciseState` holds the predict-then-run reveal toggles (answer hidden until "Reveal"). `lesson_view::render` lost its internal `ScrollArea` so the left panel scrolls prose+exercises+annotation together. `main.rs`: new fields (`known_lessons`, `ex_state`, `grade_job`, `annotation`, `grade_error`); `start_grade` spawns `rusty_host::grade` (process #2) on a `std::thread` → `mpsc` channel + `ctx.request_repaint()`, `poll_grade` drains it each frame (mirrors the PTY pattern) — a multi-second `cargo test` never freezes the UI; a "checking…" state shows meanwhile. Added `rusty-grader` as a direct app dep (the pure model; no `cargo_metadata` leak). 5 unit/headless tests (link availability, annotation renders all 4 shapes, criterion_for_exercise gate, reveal toggle, every Exercise variant renders). 17 app tests pass; full workspace green.
- **Completed:** 2026-05-22T07:20:00Z
- **Files modified:** `crates/rusty-app/src/{annotation.rs,exercise_view.rs,lesson_view.rs,main.rs,voice.rs}`, `crates/rusty-app/Cargo.toml`
- **Commit:** `867d607`

## T-405 (sprint 4, consumes backlog T-EX1)
- **Description:** Authored lesson 1's exercises + made them gradeable in-app. Added `[[exercises]]` to `foundations-01-hello/lesson.toml`: a **Worked** (smallest program), a **PredictThenRun** (`1+2` → predict `3`), a **Faded** (`cargo test`: define `greeting()` so the provided `#[cfg(test)]` test passes), and an **Open** (`cargo run` must print `I compiled my first Rust program!`) — satisfying prompt §3's ≥1-each requirement. Reworked `starter/`/`solution/` `src/main.rs`: the Faded test calls `super::greeting()`, which is **undefined in the starter** so an unedited `cargo test` yields **E0425** (unresolved name) — which `concept_for_code` maps to the loaded `foundations-01-hello`, making the **live concept-link** path reachable during the heartbeat (C-002); the `#[cfg(test)]` placement keeps `cargo run` compiling regardless. Solution defines `greeting` and prints the Open line. 6 tests: a curriculum variant-coverage assertion in `tests/content.rs`, and 5 real-cargo grade integration tests in new `tests/lesson1_grade.rs` (solution test→Pass, starter test→CompileError w/ E0425, solution run→Pass, starter run→RunMismatch, and grade→annotate→live link to lesson 1). Full workspace green.
- **Completed:** 2026-05-22T07:40:00Z
- **Files modified:** `content/lessons/foundations-01-hello/{lesson.toml,starter/src/main.rs,solution/src/main.rs}`, `crates/rusty-host/tests/{content.rs,lesson1_grade.rs}`
- **Commit:** `53a3049`
