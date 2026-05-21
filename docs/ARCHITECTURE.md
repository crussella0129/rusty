---
project: Rusty
tags: [rusty, architecture, egui, eframe, pty, lsp, stub]
related:
  - "[[Rusty]]"
  - "[[egui]]"
  - "[[eframe]]"
  - "[[PTY]]"
  - "[[LSP]]"
status: stub
---

# Rusty — Architecture

> Stub. Filled out in [[Phase 1]] (PTY spike) and refined through [[Phase 4]]
> (rust-analyzer integration), once the three-process model is real code rather
> than a plan.

[[Rusty]] is a native [[eframe]] app whose entire executor is three subprocesses
owned by `rusty-host`: a [[PTY]]-attached sandbox shell, a `cargo test
--message-format=json` grading process, and a `rust-analyzer` [[LSP]] subprocess.
The four engine crates (`rusty-curriculum`, `rusty-scheduler`, `rusty-grader`,
`rusty-pedagogy`) stay OS-portable; everything OS-dependent lives in `rusty-host`
and `rusty-terminal`. See `RUSTY_PROMPT.md` §2 for the canonical diagram.
