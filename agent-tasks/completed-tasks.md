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

## T-501 (sprint 5)
- **Description:** `Step` schema + atomic, behavior-preserving migration. Curriculum: `Lesson.body: Vec<Block>` + `exercises: Vec<Exercise>` replaced by `steps: Vec<Step>` where `Step{ #[serde(default)] blocks: Vec<Block>, exercise: Option<Exercise>, hint: Option<String> }`; `Step::is_gating()` (Some Faded/Open); pure `visible_prefix(&[Step], &[bool]) -> usize` (prefix up to & incl. the first incomplete gating step). Loader validates ≥1 step (`steps` is `#[serde(default)]` so "no steps" → a clean `Invalid`, not a serde error). App: `lesson_view::render` now iterates steps (blocks then the step's inline exercise, indexed by step) and returns the pressed Check's criterion; `exercise_view::render` (loop+"Exercises" heading) collapsed to `render_exercise(ui, idx, ex, …)`; `main` calls the new `lesson_view::render` and drops the separate exercise call; removed unused `voice::EXERCISES_HEADING`. Re-authored lesson 1 into 6 `[[steps]]` (intro+code+run+tip, recap, Worked, PredictThenRun, Faded[gating], Open[gating]). Migrated tests (loader VALID→steps incl. doubly-nested `[[steps.blocks]]` + inline criterion per C-005, zero-steps→Invalid; `lesson_view` ALL_BLOCKS fixture; host `content.rs` real-lesson + variant scan over `steps[].exercise`). **No new behavior (all steps visible, no gating yet).** Confirmed: internally-tagged enums parse from doubly-nested arrays-of-tables. 109 workspace tests green; the s4 lesson1_grade real-cargo suite unaffected.
- **Completed:** 2026-05-22T15:55:00Z
- **Files modified:** `crates/rusty-curriculum/src/{model.rs,loader.rs,lib.rs}`, `crates/rusty-app/src/{lesson_view.rs,exercise_view.rs,main.rs,voice.rs}`, `content/lessons/foundations-01-hello/lesson.toml`, `crates/rusty-host/tests/content.rs`
- **Commit:** `f8071b4`

## T-502 (sprint 5)
- **Description:** In-memory progress + gated render + grade→step wiring. `LessonProgress{completed: Vec<bool>, attempts: Vec<u32>}` (sized to `steps.len()`) in `RustyApp` with `apply(step,&Verdict)` (Pass→`completed[step]=true`; else `attempts[step]+=1`), `all_complete()`, `completed()`. `lesson_view::render` now takes `&LessonProgress`, renders only `steps[..visible_prefix(steps, completed)]` (so a gating step hides everything after it until it passes), returns `Option<(usize, SuccessCriterion)>` (the pressed step's index), and reveals the recall prompt + further-reading wrap-up once `all_complete()`. `main`: `start_grade(step, criterion, ctx)` records `pending_step`; `poll_grade` folds the verdict into `progress.apply(step, …)` before mapping the annotation; `grade_outcome` now borrows `&Result`. New voice headings RECALL/FURTHER_READING. 7 new tests (apply pass/fail, all_complete predicate, gated-render hides-past-gate + visible_prefix==2, complete-progress renders recall/further-reading; grade_outcome borrow-updated). 113 workspace tests green; clippy/fmt clean.
- **Completed:** 2026-05-22T16:10:00Z
- **Files modified:** `crates/rusty-app/src/{main.rs,lesson_view.rs,voice.rs}`
- **Commit:** `e750e13`

## T-503 (sprint 5)
- **Description:** Reveal animation. In `lesson_view::render`, each visible step is wrapped in `ui.scope(|ui| { ui.multiply_opacity(factor); … })` where `factor = ctx.animate_bool_with_time(Id::new(("rusty_step_reveal", i)), true, 0.35)` — a stable per-step id with a `true` target ramps 0→1 the first time the step appears (already-visible steps sit at 1), so a newly-revealed step (after a gate passes) fades in instead of popping. The scoped closure returns that step's Check request so the index↔criterion plumbing is preserved. 1 new test (`test_lesson_pane_animates_without_panic`: two frames on one `Context` so the animation advances). 28 app tests; clippy/fmt clean.
- **Completed:** 2026-05-22T16:20:00Z
- **Files modified:** `crates/rusty-app/src/lesson_view.rs`
- **Commit:** `5181d7b`

## T-504 (sprint 5)
- **Description:** Tips + lesson-1 hints + lesson-complete flourish. Pure `lesson_view::tip_visible(step, attempts) -> bool` (gating step + `hint.is_some()` + `attempts >= 1`); the tip renders inside the gating step's frame (amber `voice::TIP_LABEL` + the hint markdown) after the first failed Check, reading `progress.attempts(step)` (accessor re-added to `LessonProgress`). When `all_complete()`, a green H2 `voice::LESSON_COMPLETE` flourish renders above the recall + further-reading wrap-up. Authored lesson 1's Faded step `hint` (points the learner at defining `greeting`). 2 new app tests (`test_tip_visible_predicate`; `test_tip_hidden_then_shown_render` — attempts 0 hidden → after a failed `apply` shown, both no-panic) + host `content.rs` now asserts ≥1 gating step has a hint. Full workspace green; clippy/fmt clean.
- **Completed:** 2026-05-22T16:35:00Z
- **Files modified:** `crates/rusty-app/src/{lesson_view.rs,voice.rs,main.rs}`, `content/lessons/foundations-01-hello/lesson.toml`, `crates/rusty-host/tests/content.rs`
- **Commit:** `f88b9fc`

## T-601 (sprint 6)
- **Description:** ▶ run prompt is now an interactive command. `lesson_view::render` returns a new `pub struct LessonAction { check: Option<(usize, SuccessCriterion)>, run: Option<String> }` (replaces the bare `Option<(usize, criterion)>`); `render_block` returns `Option<String>` (the run command if clicked, else None); the NowRun branch renders as `ui.add(egui::Button::new(text).frame(false))` — `frame(false)` keeps the existing hyperlink-blue monospace strong run-prompt visual while gaining real click+hover affordance. The loop aggregates per-step actions into a single `LessonAction`. `main.rs` ui handler: on `action.check` calls `start_grade` (unchanged); on `action.run` writes `{cmd}\r` into `self.session` (the embedded PTY), so the shell executes the command. Pure `run_request_for_block(&Block) -> Option<String>` extracted for testing. 3 new app tests (run_request_for_block NowRun + non-NowRun, LessonAction::default is no-op); existing render tests still green. 34 app tests; clippy/fmt clean.
- **Completed:** 2026-05-23T19:20:00Z
- **Files modified:** `crates/rusty-app/src/{lesson_view.rs,main.rs}`
- **Commit:** `60d6fd7`

## T-602 (sprint 6)
- **Description:** Diagnostic eprintln traces + `enforce_gradeable_step` panic-guard for the Reveal-Pass mystery bug. Added stderr traces (prefix `[rusty-trace]`) at every grade-trigger seam: `start_grade` entry, `poll_grade.ok` (with pending_step + verdict), PredictThenRun Reveal click, Faded/Open Check click in render_exercise, and the `pair_check` site in lesson_view::render (with the exercise variant name). Added a pure free fn `enforce_gradeable_step(&Lesson, step)` in main.rs that panics with the step index + actual variant name if the step's exercise isn't Faded/Open; called at the single chokepoint in `ui()` immediately before `start_grade`. 4 new app tests (panic for Worked/PredictThenRun via `#[should_panic(expected = "not gradeable")]`; ok for Faded/Open). 38 app tests; clippy/fmt clean. Heartbeat next: user reproduces Reveal-Pass in instrumented build and shares `[rusty-trace]` lines.
- **Completed:** 2026-05-23T19:35:00Z
- **Files modified:** `crates/rusty-app/src/{main.rs,exercise_view.rs,lesson_view.rs}`
- **Commit:** `57262c3` (+ fix-up commit mirroring traces to `$TEMP/rusty-trace.log` + panic-hook, since Windows GUI-subsystem stderr buffering swallowed the original eprintln output on a hard-exit)

## T-603 (sprint 6)
- **Description:** Bug closed as **not reproducing under instrumentation**. User reproduced the click sequence (Reveal → Check) in the T-602 instrumented build with a cleared sandbox; the trace log shows a clean sequence — `reveal step=3` fires alone (no `pair_check`, no `start_grade`, no spurious Pass), and the subsequent Check fires normally on step 4 (Faded) yielding the expected E0425 CompileError. Hypothesis: **T-601's refactor (lesson_view::render returning `LessonAction` instead of bare `Option<(usize, criterion)>`) inadvertently changed the control flow enough to dodge the original quirk**. Without a confirmed cause, the structural defence remains: `enforce_gradeable_step` and its 4 tests stay as the permanent guard against silent recurrence. **Removed:** the `trace()` helper, `trace_log_path()`, the panic hook in `main()`, the `std::io::Write` import, and the five `trace(...)` call sites (in main.rs poll_grade/start_grade; exercise_view.rs reveal/check; lesson_view.rs pair_check). `cargo test -p rusty-app` → 38 pass; grep for stale `rusty-trace`/`trace(`/`trace_log` references → empty. Clippy/fmt clean.
- **Completed:** 2026-05-23T20:05:00Z
- **Files modified:** `crates/rusty-app/src/{main.rs,exercise_view.rs,lesson_view.rs}`
- **Commit:** `979f5c3`

## T-604 (sprint 6)
- **Description:** Fade-in for the PredictThenRun reveal — Output/explanation no longer "snap" when the learner clicks Reveal. `render_exercise`'s PredictThenRun branch now calls `ui.ctx().animate_bool_with_time(Id::new(("rusty_reveal_fade", i)), state.revealed(i), 0.35)` **every frame** (with target = current reveal state), so the stored animation value tracks the false→true transition on Reveal-click and ramps 0→1 over ~0.35s; when revealed, the Output/explanation render inside `ui.scope` with `multiply_opacity(factor)`. Same pattern as the T-503 step-reveal fade. 1 new app test (`test_predict_reveal_animates_without_panic` — two frames on one `Context` so the animation actually transitions; asserts the stored factor stays in `[0,1]`). 39 app tests; full workspace 125 green; clippy/fmt clean.
- **Completed:** 2026-05-23T20:15:00Z
- **Files modified:** `crates/rusty-app/src/exercise_view.rs`
- **Commit:** `1e88615`

## T-701 (sprint 7)
- **Description:** Marker-file idempotency in `prepare_sandbox` (checks `Cargo.toml`, `src/main.rs`, and parsed `[workspace]` key to identify a healthy sandbox and prevent outer workspace escalation). Integrates a startup sandbox health check in `RustyApp::new` and implements a safe `fallback_sandbox()` routing to OS temp dir when sandbox preparation fails.
- **Completed:** 2026-06-05T04:00:23Z
- **Files modified:** `crates/rusty-host/src/content.rs`, `crates/rusty-host/Cargo.toml`, `crates/rusty-app/src/main.rs`, `crates/rusty-host/tests/content.rs`
- **Commit:** `149bde2`

## T-702 (sprint 7)
- **Description:** Cargo manifest locking for the background grader. Spawns `cargo test` and `cargo run` with `--manifest-path <sandbox>/Cargo.toml`, ensuring workspace resolution is explicitly locked to the sandboxed lesson project folder and cannot silently escalate to the root repo manifest.
- **Completed:** 2026-06-05T04:00:23Z
- **Files modified:** `crates/rusty-host/src/grade.rs`, `crates/rusty-host/tests/grade.rs`
- **Commit:** `db03ca0`

## T-703 (sprint 7)
- **Description:** Perceptible reveal fade-in animation tuning. Increased the PredictThenRun explanation fade-in duration from 0.35s to 0.6s to make the visual transition smoother and more obvious to learners.
- **Completed:** 2026-06-05T04:00:56Z
- **Files modified:** `crates/rusty-app/src/exercise_view.rs`
- **Commit:** `bfba9d0`

## T-801 (sprint 8)
- **Description:** Workspace dependencies and crate wiring (add `lsp-types` and `serde_json` to root workspace manifest and to dependencies of `rusty-host` and `rusty-app`).
- **Completed:** 2026-06-07T05:27:30Z
- **Files modified:** `Cargo.toml`, `crates/rusty-host/Cargo.toml`, `crates/rusty-app/Cargo.toml`
- **Commit:** `ce2f54e`

## T-802 (sprint 8)
- **Description:** JSON-RPC message framing (implemented `write_message` and `read_message` for LSP framing, along with tests, in `rusty-host::lsp`).
- **Completed:** 2026-06-07T05:30:15Z
- **Files modified:** `crates/rusty-host/src/lsp.rs`, `crates/rusty-host/src/lib.rs`
- **Commit:** `f399d86`

## T-803 to T-809 (sprint 8)
- **Description:** Implement LSP integration (`rust-analyzer`). Spawns `rust-analyzer` and initializes the handshake (T-803), synchronizes files and receives diagnostics (T-804), maps asynchronous request/responses for hover and completion (T-805), integrates into `rusty-app` lifecycle and `Editor` (T-806), renders live error underlines and gutter diagnostics (T-807), maps UI pointer hit-testing to LSP coordinates for hover tooltips (T-808), and intercepts keyboard triggers to render an autocomplete popup (T-809).
- **Completed:** 2026-06-09T10:19:13-04:00
- **Files modified:** `crates/rusty-app/src/{main.rs, editor.rs}`, `crates/rusty-host/src/lsp.rs`, `crates/rusty-host/tests/lsp_integration.rs`
- **Commit:** `edb89ab`

## T-901 to T-904 (sprint 9)
- **Description:** Implement Phase 5 (Recall, scheduling, persistence). Added an OS-portable SM-2 spaced repetition engine in \usty-scheduler\. Added persistent state saving to \.rusty-state/progress.json\ in \usty-app\. Refactored the recall prompt UI to be interactive, accepting selections and short answers, grading them, and applying SM-2 updates. Implemented the "Due Reviews" startup mode to enforce learning retention before advancing to new lessons.
- **Completed:** 2026-06-09T10:54:15-04:00
- **Files modified:** \crates/rusty-scheduler/src/lib.rs\, \crates/rusty-app/src/{main.rs, state.rs, lesson_view.rs}\, \crates/rusty-curriculum/src/model.rs\
- **Commit:** \913437\


## T-1001 to T-1004 (sprint 10)
- **Description:** Implement Phase 6 (Lesson 2 content) and SM-2 progression refactor. Refactored the usty-scheduler logic to use lesson-based intervals instead of real-world days. Added lesson tracking to usty-app state. Authored Lesson 2 (oundations-02-variables) including starter/solution cargo projects, Faded/Open/PredictThenRun exercises on mutability and shadowing, and an interactive RecallPrompt.
- **Completed:** 2026-06-09T11:15:00-04:00
- **Files modified:** crates/rusty-scheduler/src/lib.rs, crates/rusty-app/src/{main.rs, state.rs}, content/lessons/foundations-02-variables/*
- **Commit:** 35b0530


## T-1101 to T-1106 (sprint 11)
- **Description:** Implement Phase 6 (Lesson 3 content). Authored Lesson 3 (\oundations-03-ownership\) focusing on Move Semantics and the Copy Trait. Built starter/solution projects with exercises for fixing 'use of moved value' errors using \.clone()\.
- **Completed:** 2026-06-09T12:35:00-04:00
- **Files modified:** \crates/rusty-app/src/main.rs\, \content/lessons/foundations-03-ownership/*\
- **Commit:** \4c06df3\


## T-1201 to T-1206 (sprint 12)
- **Description:** Implement Phase 6 (Lesson 4 content). Authored Lesson 4 (\oundations-04-borrows\) focusing on Borrowing rules (1 mutable XOR N immutable) and Lifetime syntax (\<'a>\). Built starter/solution projects with exercises for fixing simultaneous mut/immut borrows and missing lifetime annotations.
- **Completed:** 2026-06-09T21:28:00-04:00
- **Files modified:** \crates/rusty-app/src/main.rs\, \content/lessons/foundations-04-borrows/*\
- **Commit:** \aa27eb\ and \27e6d17\

