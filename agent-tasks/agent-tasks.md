# Agent Tasks (Persistent Backlog)


- [ ] T-FILETREE (partially done in s4 T-403 → a flat `.rs` file picker; a real nested tree view still pending): sandbox file-tree view in the workspace pane. (From s2 critique C-008; prompt §6 Phase 2.)

- [ ] T-PORT-GUARD (deferred → Phase 1 / sprint 1): add an automated portability guard (e.g. `cargo-deny` rule or `#![forbid]`-style contract) preventing `std::process`/OS-FS/PTY deps from leaking into the four portable crates (rusty-curriculum, rusty-scheduler, rusty-grader, rusty-pedagogy). Cheap to add once real code exists. — touches: `deny.toml` or crate-level lints. (From s0 plan critique C-006.)

<!-- T-EX1 completed in sprint 4 (T-405). -->
<!-- Sprint 4 (Phase 3b) build tasks T-401..T-405 all completed — see completed-tasks.md. -->

## Sprint 5 (Progressive content disclosure)
- [ ] T-504: tips (hint after first fail) + lesson-1 hints + lesson-complete flourish. — touches: `crates/rusty-app/src/{lesson_view.rs,voice.rs}`, `content/lessons/foundations-01-hello/lesson.toml`.
