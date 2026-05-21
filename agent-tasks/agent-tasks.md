# Agent Tasks (Persistent Backlog)

- [ ] T-204 (sprint 2): author foundations-01-hello (lesson.toml + starter/ + solution/, [workspace] detach) — touches: `content/lessons/foundations-01-hello/**`
- [ ] T-205 (sprint 2): pulldown-cmark→egui markdown + lesson_view renderer — touches: `Cargo.toml`, `crates/rusty-app/{Cargo.toml,src/markdown.rs,src/lesson_view.rs,src/main.rs}`
- [ ] T-206 (sprint 2): load+render lesson 1, PTY in lesson sandbox, fallible new() — touches: `crates/rusty-app/src/{main.rs,voice.rs}`

- [ ] T-EX1 (deferred → Phase 3): author lesson 1 exercises (Worked/Faded/Open + PredictThenRun) per §3/§4. (From s2 critique C-007.)
- [ ] T-FILETREE (deferred → Phase 3): sandbox file-tree view in the workspace pane, bundled with the editor. (From s2 critique C-008; prompt §6 Phase 2.)

- [ ] T-PORT-GUARD (deferred → Phase 1 / sprint 1): add an automated portability guard (e.g. `cargo-deny` rule or `#![forbid]`-style contract) preventing `std::process`/OS-FS/PTY deps from leaking into the four portable crates (rusty-curriculum, rusty-scheduler, rusty-grader, rusty-pedagogy). Cheap to add once real code exists. — touches: `deny.toml` or crate-level lints. (From s0 plan critique C-006.)
