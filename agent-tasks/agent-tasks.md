# Agent Tasks (Persistent Backlog)


- [ ] T-FILETREE (partially done in s4 T-403 → a flat `.rs` file picker; a real nested tree view still pending): sandbox file-tree view in the workspace pane. (From s2 critique C-008; prompt §6 Phase 2.)

- [ ] T-PORT-GUARD (deferred → Phase 1 / sprint 1): add an automated portability guard (e.g. `cargo-deny` rule or `#![forbid]`-style contract) preventing `std::process`/OS-FS/PTY deps from leaking into the four portable crates (rusty-curriculum, rusty-scheduler, rusty-grader, rusty-pedagogy). Cheap to add once real code exists. — touches: `deny.toml` or crate-level lints. (From s0 plan critique C-006.)

<!-- T-EX1 completed in sprint 4 (T-405). -->
<!-- Sprint 4 (Phase 3b) build tasks T-401..T-405 all completed — see completed-tasks.md. -->

## Sprint 5 (Progressive content disclosure)
<!-- Sprint 5 build tasks T-501..T-504 all completed — see completed-tasks.md. -->

## Sprint 6 (Bug triage: Reveal-Pass + ▶ run UX)
<!-- T-601 completed; see completed-tasks.md. -->
<!-- T-602 completed; see completed-tasks.md. -->
<!-- T-603 completed (bug not reproducing under instrumentation; T-601's refactor likely shooed it away; instrumentation removed, enforce_gradeable_step retained as permanent guard). -->
<!-- T-604 completed; see completed-tasks.md. -->

## Sprint 7 (real fix: sandbox marker + cargo manifest-path lock)
<!-- T-701 completed; see completed-tasks.md. -->
<!-- T-702 completed; see completed-tasks.md. -->
<!-- T-703 completed; see completed-tasks.md. -->

## Sprint 8 (LSP and rust-analyzer)
- [ ] T-802: JSON-RPC message framing

- [ ] T-803: Spawn rust-analyzer and initialize handshake
- [ ] T-804: File synchronization and diagnostics reception
- [ ] T-805: Async request-response matching (hover & completion)
- [ ] T-806: App integration and session lifecycle
- [ ] T-807: Editor Diagnostics rendering (underlines + gutter)
- [ ] T-808: Editor Hover hit-testing and tooltip
- [ ] T-809: Autocomplete popup and keyboard event interception

