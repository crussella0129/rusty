# Agent Tasks (Persistent Backlog)


- [ ] T-EX1 (deferred → Phase 3): author lesson 1 exercises (Worked/Faded/Open + PredictThenRun) per §3/§4. (From s2 critique C-007.)
- [ ] T-FILETREE (deferred → Phase 3): sandbox file-tree view in the workspace pane, bundled with the editor. (From s2 critique C-008; prompt §6 Phase 2.)

- [ ] T-PORT-GUARD (deferred → Phase 1 / sprint 1): add an automated portability guard (e.g. `cargo-deny` rule or `#![forbid]`-style contract) preventing `std::process`/OS-FS/PTY deps from leaking into the four portable crates (rusty-curriculum, rusty-scheduler, rusty-grader, rusty-pedagogy). Cheap to add once real code exists. — touches: `deny.toml` or crate-level lints. (From s0 plan critique C-006.)

## Sprint 4 (Phase 3b — editor + on-screen diagnostics + lesson-1 exercises)
- [ ] T-405 (consumes backlog T-EX1): author lesson-1 exercises (Worked/Faded/Open/PredictThenRun) + gradeable starter/solution. — touches: `content/lessons/foundations-01-hello/**`, curriculum test.
