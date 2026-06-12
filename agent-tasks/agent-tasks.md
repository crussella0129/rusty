# Agent Tasks (Persistent Backlog)


- [x] T-FILETREE (partially done in s4 T-403 → a flat `.rs` file picker; a real nested tree view still pending): sandbox file-tree view in the workspace pane. (From s2 critique C-008; prompt §6 Phase 2.)

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
<!-- Sprint 8 build tasks T-801..T-809 all completed — see completed-tasks.md. -->

## Sprint 9 (Phase 5: Recall, scheduling, persistence)
<!-- Sprint 9 build tasks T-901..T-904 all completed — see completed-tasks.md. -->

## Sprint 10 (Phase 6: Lesson 2 content & SM-2 progression refactor)
<!-- Sprint 10 build tasks T-1001..T-1004 all completed — see completed-tasks.md. -->

## Sprint 11 (Phase 6: Lesson 3 content - Ownership)
<!-- Sprint 11 build tasks T-1101..T-1106 all completed — see completed-tasks.md. -->

## Sprint 12 (Phase 6: Lesson 4 content - Borrows)
<!-- Sprint 12 build tasks T-1201..T-1206 all completed — see completed-tasks.md. -->

## Sprint 13 (Phase 6: Lesson 5 content - Structs)
<!-- Sprint 13 build tasks T-1301..T-1306 all completed — see completed-tasks.md. -->

## Sprint 14 (Phase 6: Lesson 6 content - Enums)
<!-- Sprint 14 build tasks T-1401..T-1406 all completed — see completed-tasks.md. -->

## Sprint 15 (Phase 6: Lesson 7 content - Errors)
<!-- Sprint 15 build tasks T-1501..T-1506 all completed — see completed-tasks.md. -->

## Sprint 16 (Phase 6: Lesson 8 content - Collections & File I/O)
<!-- Sprint 16 build tasks T-1601..T-1606 all completed — see completed-tasks.md. -->

## Sprint 17 (Phase 7: Bootstrap + Polish + Ship)
<!-- Sprint 17 build tasks T-1701..T-1706 all completed — see completed-tasks.md. -->

## Sprint 18 (Advanced Diffing & File I/O in Lesson 8)
<!-- Sprint 18 build tasks T-1801..T-1805 all completed — see completed-tasks.md. -->

## Sprint 19 (Intermediate Lesson 1 - Traits & Generics)
- [x] T-1901: Create `intermediate-01-traits` lesson directory structure (`lesson.toml`, `starter`, `solution`).
- [x] T-1902: Author `lesson.toml` with concepts, prose, and exercises focusing on an `Operator` trait.
- [x] T-1903: Write the `starter` project with `// TODO` markers for faded exercises.
- [x] T-1904: Write the complete `solution` project.

## Sprint 20 (Intermediate Lesson 2 - Iterators & Closures)
- [ ] T-2001: Implement `intermediate-02-iterators` leveraging functional parsing of datasets.

## Sprint 21 (Intermediate Lesson 3 - Lifetimes & References)
- [ ] T-2101: Implement `intermediate-03-lifetimes` building zero-copy Parsers.

## Sprint 22 (Intermediate Lesson 4 - Smart Pointers & Shared State)
- [ ] T-2201: Implement `intermediate-04-smart-pointers` building recursive AST nodes.

## Sprint 23 (Intermediate Lesson 5 - Serialization & Parsing)
- [ ] T-2301: Implement `intermediate-05-parsing` using serde and toml for prompt element headers.

## Sprint 24 (Intermediate Lesson 6 - Advanced Diffing Climax)
- [ ] T-2401: Implement `intermediate-06-diffing` (full +/- diffing with positional change awareness).
