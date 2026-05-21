# Agent Tasks (Persistent Backlog)

- [ ] T-304 (sprint 3): rusty-host run_cargo_test/run + grade() orchestrator — touches: `crates/rusty-host/{Cargo.toml,src/grade.rs,src/lib.rs,tests/grade.rs}`

- [ ] T-EX1 (deferred → Phase 3): author lesson 1 exercises (Worked/Faded/Open + PredictThenRun) per §3/§4. (From s2 critique C-007.)
- [ ] T-FILETREE (deferred → Phase 3): sandbox file-tree view in the workspace pane, bundled with the editor. (From s2 critique C-008; prompt §6 Phase 2.)

- [ ] T-PORT-GUARD (deferred → Phase 1 / sprint 1): add an automated portability guard (e.g. `cargo-deny` rule or `#![forbid]`-style contract) preventing `std::process`/OS-FS/PTY deps from leaking into the four portable crates (rusty-curriculum, rusty-scheduler, rusty-grader, rusty-pedagogy). Cheap to add once real code exists. — touches: `deny.toml` or crate-level lints. (From s0 plan critique C-006.)
