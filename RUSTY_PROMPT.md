---
project: Rusty
owner: Charles Russella
created: 2026-05-21
updated: 2026-05-21
version: 2
supersedes: RUSTY_PROMPT.md (v1)
tags: [rust, egui, eframe, pedagogy, claude-code, prompt, learning, native-app, pty, tescellate-adjacent]
related:
  - "[[Tescellate]]"
  - "[[Carbide]]"
  - "[[Rust]]"
  - "[[Sprint Loops]]"
  - "[[Claude Thoughts Vault]]"
status: draft-spec
sprint_phase: 0-spec
---

# Rusty v2 — Claude Code Build Prompt

**A 100% Rust native desktop application that teaches Rust by guiding learners through real `cargo` invocations inside an embedded terminal, against a sandboxed workspace of real cargo projects.**

**Stack:** Rust → emilk/egui (via eframe) → native binary. No WASM in Rusty's runtime. No backend. No website. Distribution is GitHub clone + bootstrap script.

> Paste this entire file into Claude Code as the initial prompt. Read it fully, ask clarifying questions in ONE batch, then execute the phased plan. Do not skip phases. Do not stub anything out unless explicitly noted as deferred.

> **This is v2 and supersedes v1 entirely.** v1 targeted a browser-first WASM deployment with three competing executor strategies; v2 is a native app that shells out to the learner's real Rust toolchain. Section 12 (mascot/voice) is the only major part carried forward unchanged.

---

## 0. Identity and Constraints

### Project name
**Rusty** — named after the user's dog (large, orange). See Section 12 for the mascot and voice spec; it constrains the UI.

### Distribution model: GitHub-native

The repository **is** the product. No hosted website, no download page, no auto-update mechanism, no app-store presence. Learners get Rusty by:

```bash
git clone https://github.com/<owner>/rusty.git
cd rusty
./install.sh        # or .\install.ps1 on Windows
```

`install.sh` does three things, in order:

1. **Detects** the Rust toolchain (`rustup`, `rustc`, `cargo`). If absent, **prompts** the user (interactive y/N) to install it via the official rustup one-liner: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`. Never installs Rust silently. If the learner declines, prints clean manual instructions and exits cleanly.
2. **Builds Rusty from source** via `cargo build --release`. The first compile is Rusty's first lesson before the lessons even start: the learner watches their freshly installed toolchain compile a real Rust project.
3. **Launches** the resulting binary at `./target/release/rusty`.

The bootstrap is **a pedagogical artifact, not a setup step.** It is the learner's first contact with the Rust toolchain and should feel that way — narrated and paced, not silent.

"Updating Rusty" means `git pull && cargo build --release`. That is the entire update story.

### Hard technical constraints

Non-negotiable. Flag any tension before resolving.

1. **100% Rust.** No JavaScript, no Python, no Node. The install script is bash/PowerShell (unavoidable for bootstrap), but every other line of code is Rust. Once Rust is installed, all further setup work happens in cargo.
2. **`egui` for UI**, via `eframe` as the application shell. Both are from emilk on GitHub. Native target only.
3. **Native desktop application.** macOS (Intel + Apple Silicon), Linux (x86_64), Windows (x86_64). No browser build.
4. **Rust toolchain is a hard runtime dependency.** Rusty assumes `rustc`, `cargo`, and `rust-analyzer` are present on PATH at runtime. The install script ensures this.
5. **Workspace is fully sandboxed.** All learner work happens inside `<rusty-repo>/workspace/lessons/<lesson-id>/`. The embedded terminal's `cwd` is locked inside this tree. `cd` above the sandbox root is intercepted and refused with a friendly message. Rusty is not a general-purpose shell.
6. **Embedded PTY terminal.** Real pseudo-terminal, real ANSI rendering, real interactivity. This is the single most technically risky piece of the build. Spike it in Phase 1.
7. **No telemetry, no analytics, no network calls at runtime.** The install script may invoke rustup with consent; the running app makes zero network requests.
8. **MSRV:** latest stable at install time, pinned in `rust-toolchain.toml`.

### Stretch constraints

- **Cold-start to first interactive lesson: < 3 seconds** after the initial release build.
- **Single binary.** No dynamic libs beyond OS-provided ones. The `rusty` executable finds its lessons via a known relative path from the binary.
- **Reproducible builds.** Commit `Cargo.lock`. CI verifies clean build on all three OSes.

### Explicitly NOT in MVP

- No browser/WASM build of Rusty itself.
- No hosted version.
- No multi-user features, accounts, sharing, social, or sync.
- No payment, monetization, or telemetry.
- No mobile UI.
- No AI tutor / LLM integration. (Phase 6+ at earliest.)
- No content beyond the lessons in Section 4.
- No "leave the sandbox" terminal mode.
- No auto-updater — `git pull` is the update mechanism.

---

## 1. Pedagogical Thesis

Rusty operationalizes five research-backed learning principles that most programming tutorials violate.

### 1.1 Retrieval practice over re-reading
**Source:** Roediger & Karpicke (2006), *Test-Enhanced Learning: Taking Memory Tests Improves Long-Term Retention*. Psychological Science, 17(3).

Active recall produces dramatically better long-term retention than repeated study.

**Application:** Every lesson ends with a recall prompt *before* any exercise. Short-answer or multiple-choice on the lesson's key claim, graded immediately. No skipping.

### 1.2 Spaced repetition
**Source:** Cepeda, Pashler, Vul, Wixted, & Rohrer (2006), *Distributed Practice in Verbal Recall Tasks: A Review and Quantitative Synthesis*. Psychological Bulletin, 132(3).

Spacing reviews over expanding intervals produces 2–3× better retention than massed practice.

**Application:** Simplified SM-2 scheduler. Each lesson has 3–5 "atomic concepts" tracked individually. Each session opens with due reviews before new material.

### 1.3 Worked examples → faded scaffolding → independent practice
**Source:** Sweller (1988), *Cognitive Load During Problem Solving*. Cognitive Science, 12. Also Renkl & Atkinson (2003) on faded examples and the expertise reversal effect.

Novices learn far more from studying worked examples than from solving unaided. The benefit reverses with expertise. The bridge is *faded* examples.

**Application:** Every exercise comes as a triple — `Worked` (read and annotate), `Faded` (fill in 1–3 blanks in real source files), `Open` (write from scratch). Advancement is performance-driven. Faded exercises are real `src/main.rs` files with `// TODO` markers the learner edits in place.

### 1.4 Productive failure with immediate compiler feedback
**Source:** Kapur (2008), *Productive Failure*. Cognition and Instruction, 26(3). Shute (2008), *Focus on Formative Feedback*, on immediate vs. delayed feedback.

Struggling productively *before* receiving instruction improves transfer. Once struggling, feedback should be immediate.

**Application:** Some exercises present the problem before the explanation. The learner attempts a solution, sees real `rustc` output, and only then reads the framing that turns the error into insight.

### 1.5 Real-tool learning (the vimtutor principle)
**Source:** Brown, Collins, & Duguid (1989), *Situated Cognition and the Culture of Learning*. Educational Researcher, 18(1). Lave & Wenger (1991), *Situated Learning: Legitimate Peripheral Participation*.

Tools learned via simulators or sandboxes often don't transfer to the real tool. Tools learned *with* the real tool, in a guided context, do.

**Application:** Every command the learner types is a real command, executed by a real shell, against a real cargo project, producing real output. Rusty does not paraphrase or simulate. Its role is to *frame* the real tool, not replace it. This is the `vimtutor` model: `vimtutor` teaches Vim by *being* Vim with a curated text file loaded. Rusty teaches Rust by *being* a real cargo workspace with curated lessons loaded.

### 1.6 The Rust-specific bet

Rust's compiler is the best feedback teacher in mainstream languages. Most Rust tutorials waste this by paraphrasing errors instead of showing them. Rusty does the opposite: **`rustc`'s actual output is the primary teaching surface.** Lessons are scaffolding around real compiler conversations, displayed in a real terminal pane, annotated by Rusty in a side panel.

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                       Rusty (eframe, native)                          │
│                                                                        │
│  ┌──────────────────────────┐   ┌──────────────────────────────┐   │
│  │   Lesson pane (left)     │   │   Workspace pane (right)     │   │
│  │                          │   │                              │   │
│  │   - prose                │   │   ┌──────────────────────┐  │   │
│  │   - recall prompts       │   │   │   File tree         │  │   │
│  │   - "now run →           │   │   ├──────────────────────┤  │   │
│  │       cargo run"         │   │   │   Code editor       │  │   │
│  │   - annotation of last   │   │   │   (real files,       │  │   │
│  │     terminal output      │   │   │    real on-disk      │  │   │
│  │   - hint button          │   │   │    edits)            │  │   │
│  │                          │   │   ├──────────────────────┤  │   │
│  │                          │   │   │   Embedded terminal │  │   │
│  │                          │   │   │   (PTY → real shell │  │   │
│  │                          │   │   │    in sandbox dir)  │  │   │
│  └──────────────────────────┘   │   └──────────────────────┘  │   │
│                                  └──────────────────────────────┘   │
│                                                                        │
│  Sandbox: <rusty-repo>/workspace/lessons/<lesson-id>/                │
│  ↑ a real cargo project on disk: real Cargo.toml, real src/         │
└─────────────────────────────────────────────────────────────────────┘
        │                      │                       │
        │ spawns               │ reads/writes          │ parses
        ▼                      ▼                       ▼
  real `cargo`,            real files in         `cargo --message-
  `rustc`,                 workspace/lessons/    format=json` stream
  `rust-analyzer`                                 (for annotations)
```

### Crate layout

```
rusty/
├── Cargo.toml                      # workspace
├── rust-toolchain.toml
├── install.sh                      # bootstrap (Unix)
├── install.ps1                     # bootstrap (Windows)
├── crates/
│   ├── rusty-app/                  # eframe binary
│   ├── rusty-curriculum/           # typed lesson model + loader
│   ├── rusty-scheduler/            # SM-2-lite spaced repetition
│   ├── rusty-grader/               # cargo-test-based grading + AST hints
│   ├── rusty-pedagogy/             # research citations + helpers
│   ├── rusty-host/                 # PTY, cargo subprocess, file I/O
│   └── rusty-terminal/             # ANSI/VT100 rendering on egui
├── content/
│   └── lessons/
│       └── foundations-01-hello/
│           ├── lesson.toml         # metadata, prose, recall, exercises
│           ├── starter/            # cargo project as it begins
│           │   ├── Cargo.toml
│           │   └── src/main.rs
│           └── solution/           # for hint reveal + grader reference
│               └── src/main.rs
├── workspace/                      # learner sandbox (gitignored)
│   └── lessons/                    # populated on first open per lesson
├── assets/                         # fonts, mascot SVGs
└── docs/
    ├── ARCHITECTURE.md
    ├── PEDAGOGY.md
    ├── CONTENT_AUTHORING.md
    └── INSTALL.md
```

### The three subprocess relationships

The architecture rests on three subprocess interactions, all owned by `rusty-host`:

**1. PTY-attached shell.** A persistent shell process wired to a terminal widget in the egui UI. `cwd` locked to `<rusty-repo>/workspace/lessons/<current-lesson>/`. The learner can type any command, but `cd ..` past the sandbox root is intercepted and refused with a friendly message.

Use the `portable-pty` crate for cross-platform PTY (abstracts macOS/Linux unix pty and Windows ConPTY). For ANSI rendering, evaluate `egui_term` first; if it's not mature enough at build time, roll a thin VT100 renderer using `vte` for escape-sequence parsing and egui's text widgets for rendering. Decide in the Phase 1 spike.

**2. `cargo` invocations for grading.** When the learner clicks "Check," Rusty spawns `cargo test --message-format=json` in the sandbox, captures the JSON diagnostic stream, parses it via the `cargo_metadata` crate, and runs the grader against the result. Separate subprocess from the PTY shell; invisible to the learner.

**3. `rust-analyzer` as an LSP subprocess.** Spawned once per session, kept alive, talked to over JSON-RPC. Rusty consumes diagnostics, hover info, and completions, and renders them inside the egui code editor — squiggles in the gutter, type info on hover, autocomplete on `.`. This is what makes editing feel alive *before* the learner ever hits Run.

This three-process model is the entire executor story. There is no `SynPatternExecutor`, no rubrc, no hosted endpoint, no rust-in-WASM gymnastics. The compiler is on disk; we just talk to it.

### Sandbox enforcement

The PTY shell's `cwd` is the learner's lesson directory. Three layers of sandbox enforcement:

1. **`cwd` is set explicitly** when the PTY is spawned and on each lesson change.
2. **`cd` command interception:** Rusty parses input lines before forwarding to the PTY. If a line starts with `cd ` and the resolved target is outside the sandbox root, it's rejected with a message: *"Rusty's terminal stays inside the lesson directory. Try `cd .` to see where you are."*
3. **Path validation on every file operation** the app does. The grader, the file tree, the editor — all refuse paths outside the sandbox.

This is sandbox-as-good-UX, not sandbox-as-security. A determined learner can escape via shell tricks; that's fine. The goal is to keep accidental wandering inside the safe zone, not to imprison.

---

## 3. Curriculum Data Model

Lessons live as `lesson.toml` + a real cargo project on disk. Loaded into typed Rust at runtime.

```rust
pub struct Lesson {
    pub id: LessonId,                    // "foundations-01-hello"
    pub title: String,
    pub track: Track,
    pub prereqs: Vec<LessonId>,
    pub estimated_minutes: u8,
    pub concepts: Vec<Concept>,          // 3–5 atomic concepts
    pub body: Vec<Block>,                // prose, code, callouts, "now run X" prompts
    pub recall_prompt: RecallPrompt,
    pub exercises: Vec<Exercise>,
    pub starter_project: PathBuf,        // relative to lesson dir
    pub solution_project: PathBuf,
    pub further_reading: Vec<Reference>,
}

pub struct Concept {
    pub id: ConceptId,                   // tracked individually for SRS
    pub claim: String,
    pub why_it_matters: String,
    pub common_misconception: Option<String>,
}

pub enum Exercise {
    Worked(WorkedExample),
    Faded(FadedExample),                 // edit real src/ with TODO markers
    Open(OpenChallenge),                 // write from scratch
    PredictThenRun(PredictExercise),
}

pub struct FadedExample {
    pub file_path: PathBuf,              // within sandbox
    pub todo_markers: Vec<TodoMarker>,   // line ranges, expected shape
    pub check_command: String,           // typically "cargo test"
    pub success_criterion: SuccessCriterion,
}

pub enum SuccessCriterion {
    CargoTestPasses,
    CargoRunOutputMatches(String),
    AstShape(AstRule),                   // for style enforcement
    All(Vec<SuccessCriterion>),
}
```

The exercise enum is the pedagogical commitment encoded in types. Each lesson must contain at least one of each Worked/Faded/Open variant.

---

## 4. MVP Content Scope

Build **one complete track** — *Foundations* — to validate the system. Eight lessons:

1. **Hello, compiler.** What `cargo new`, `cargo build`, `cargo run` actually do. Reading your first error.
2. **Variables, mutability, shadowing.** Why immutable-by-default.
3. **Ownership basics: move semantics.** Hardest concept; budget two lessons.
4. **Borrows and lifetimes (informal).** No `'a` syntax yet.
5. **Structs and methods.**
6. **Enums and pattern matching.** `Option<T>` introduced here.
7. **Error handling with `Result<T, E>` and `?`.**
8. **Collections: `Vec`, `String`, `HashMap`.** Iterators as teaser for next track.

Each lesson: ~5 minutes of reading, 3 exercises minimum (Worked + Faded + Open), one PredictThenRun, one recall prompt, ~10–15 minutes total.

After Foundations ships, content scales by authoring more lesson directories — no code changes.

---

## 5. Grading Model

The grader's primary mechanism is real `cargo test`. Secondary mechanisms hint at style.

### Primary: cargo test

Every Faded and Open exercise has a `tests/` directory in its starter project. The grader spawns `cargo test --message-format=json` and parses the result.

- **All tests pass** → exercise complete, advance.
- **Some tests fail** → show which, render `rustc`'s real diagnostic in the lesson pane's annotation area, suggest re-reading the relevant concept.
- **Compile error** → don't even reach tests; render the compile error prominently, annotate it with the matching concept ("this is error E0382 — see Lesson 3 on move semantics").

### Secondary: AST hints

For style and idiom feedback (not pass/fail), parse the learner's submission with `syn` and run optional checks:

- "You solved it with a loop; the idiomatic Rust solution uses `.iter().sum()`. Want to see?"
- "You used `.unwrap()`; this works but the lesson on error handling will show you why `?` is better here."

AST hints **never block advancement.** They are post-success suggestions.

### Compiler error mapping

Maintain a curated map of `rustc` error codes (E0382, E0502, E0106, etc.) to the lesson that teaches the relevant concept. When the grader sees a known error code, the annotation pane links to the lesson:

```
error[E0382]: borrow of moved value: `s`
   ↑ this is Lesson 3 ("Ownership Basics") — open it
```

The map starts small (covers Foundations errors only) and grows as tracks are added.

---

## 6. Phased Build Plan

Each phase ends with a working, demonstrable app. Don't start the next phase until the previous one runs end-to-end.

### Phase 0 — Spec lock and skeleton (target: 1 day)
- Read this prompt fully. Surface clarifying questions in **one batch**.
- Confirm three-process architecture and PTY library choice (default: `portable-pty`).
- Decide ANSI renderer: try `egui_term` first; fall back to `vte` + custom egui widget.
- Create workspace skeleton — all crates, empty libs.
- `cargo check` passes across workspace.
- Commit: `chore: workspace skeleton`.

### Phase 1 — PTY spike + sandboxed shell (target: 3–5 days)
- Implement `rusty-host`'s PTY layer: spawn a shell, attach stdin/stdout, ANSI-render output in an egui panel.
- Verify `cargo run --version` works inside the embedded terminal.
- Verify ANSI colors render correctly (`ls --color=always` is a good test).
- Implement sandbox `cwd` lock and `cd ..` interception.
- App opens, shows an empty lesson pane on the left and a working terminal on the right.
- **This phase de-risks the entire project. If PTY rendering doesn't work cleanly, stop and reconsider before Phase 2.**
- Commit: `feat(host): embedded PTY with sandbox enforcement`.

### Phase 2 — Curriculum model + lesson 1 (target: 2–3 days)
- Implement `rusty-curriculum` types and `lesson.toml` + Markdown loader.
- Author lesson 1 (`foundations-01-hello`): `lesson.toml`, `starter/`, `solution/`.
- Implement lesson renderer in the left pane: prose, code blocks, "now run →" prompts.
- On lesson open, `starter/` is copied into `workspace/lessons/foundations-01-hello/`, the terminal's `cwd` updates, the file tree displays it.
- Learner can read lesson 1 and type `cargo run` in the terminal to see output.
- Commit: `feat(curriculum): lesson model + lesson 1 renders`.

### Phase 3 — Editor + grader + diagnostics (target: 4–6 days)
- Add `egui` code editor for the workspace files. Syntax highlighting via `egui_extras::syntax_highlighting` or `syntect`.
- Implement `rusty-grader` with `cargo test --message-format=json` parsing via `cargo_metadata`.
- Build the compiler-error-to-lesson map for Foundations error codes.
- Implement the annotation pane: when grading runs, the lesson pane shows the result with concept links.
- Add three exercises to lesson 1 (Worked + Faded + Open). All completable in-app.
- Commit: `feat(grader): cargo-test grading with annotated diagnostics`.

### Phase 4 — rust-analyzer integration (target: 3–5 days)
- Spawn `rust-analyzer` as an LSP subprocess from `rusty-host`.
- Implement minimal LSP client: `textDocument/didOpen`, `didChange`, `publishDiagnostics`, `hover`, `completion`.
- Render diagnostics as gutter squiggles in the code editor.
- Render hover info on cursor hover.
- Wire completions to autocomplete on `.`.
- **This is the phase that makes Rusty feel alive.** Before it, the learner edits and waits for grading. After it, errors appear as they type.
- Commit: `feat(lsp): rust-analyzer integration with live diagnostics`.

### Phase 5 — Recall, scheduling, persistence (target: 3 days)
- Implement `RecallPrompt` rendering and grading.
- Implement `rusty-scheduler` with SM-2-lite. Concepts have `ease`, `interval_days`, `due_at`.
- Persist progress to `<rusty-repo>/.rusty-state/progress.json` (gitignored).
- Add a "Due Reviews" landing screen that appears before new lessons when reviews are due.
- Commit: `feat(scheduler): spaced repetition + persistence`.

### Phase 6 — Content fill: lessons 2–8 (target: 5–7 days)
- Author the remaining seven Foundations lessons.
- Each: `lesson.toml`, real `starter/` and `solution/` cargo projects, 3+ exercises, 1 PredictThenRun, 1 recall prompt, prereqs wired.
- Walk through the full track yourself before declaring done.
- Commit per lesson: `content(lesson-N): <title>`.

### Phase 7 — Bootstrap + polish + ship (target: 3–4 days)
- Write `install.sh` and `install.ps1`. Test on clean machines (or VMs) for all three OSes.
- Bootstrap detection logic: clean, friendly, never silent on Rust install.
- Keyboard navigation pass: every interactive element reachable without mouse.
- Mascot illustration (see Section 12).
- README with screenshots, install instructions, "what is this" framing.
- GitHub Actions: build + test on Linux/macOS/Windows on every push.
- Commit: `chore: ship MVP`.

### Phase 8+ — Deferred

- More tracks (Intermediate, Async, Macros, WASM-as-topic).
- Community-contributed lessons.
- LLM-assisted lesson hints.
- Optional stripped-down browser preview (if the curriculum/scheduler/grader crates stayed portable).

---

## 7. Reference Material

These are the resources Rusty draws from. Use as primary sources when authoring lesson content and exercises. Lesson `further_reading` fields should cite specific sections.

### Canonical Rust resources

**Books and official docs:**

- The Rust Programming Language ("the Book") — https://doc.rust-lang.org/book/
- Brown University interactive Book — https://rust-book.cs.brown.edu/ — *Itself a research-backed Rust teaching tool. Study its "ownership inventory" visualizations and quiz patterns.*
- Rust by Example — https://doc.rust-lang.org/rust-by-example/
- Rustlings — https://github.com/rust-lang/rustlings — *The closest existing thing to Rusty. Study its exercise structure and hint system. Rusty's edge over Rustlings: integrated terminal, spaced repetition, lesson scaffolding, mascot personality.*
- Programming Rust (Blandy/Orendorff/Tindall, 2nd ed., O'Reilly) — sanity check on technical claims
- Rust for Rustaceans (Gjengset) — informs Phase 8+ tracks
- Rust Atomics and Locks (Mara Bos) — https://marabos.nl/atomics/ — for the eventual concurrency track
- The Rust Reference — https://doc.rust-lang.org/reference/
- The Rustonomicon — https://doc.rust-lang.org/nomicon/
- The Cargo Book — https://doc.rust-lang.org/cargo/
- The Little Book of Rust Macros — https://veykril.github.io/tlborm/
- Rust Design Patterns — https://rust-unofficial.github.io/patterns/
- Crafting Interpreters (Nystrom) — https://craftinginterpreters.com/ — *Lesson design inspiration; Nystrom's progressive disclosure is the bar.*

**Reading + blogs:**

- This Week in Rust — https://this-week-in-rust.org/
- Without Boats — https://without.boats/
- fasterthanli.me (Amos) — https://fasterthanli.me/
- Niko Matsakis — https://smallcultfollowing.com/babysteps/

### Repositories to study

**For learn-by-doing patterns:**
- rust-lang/rustlings — https://github.com/rust-lang/rustlings — exercise curve, hint system, completion detection
- rust-lang/rust-by-example — https://github.com/rust-lang/rust-by-example — minimal-example pedagogy

**For idiomatic Rust to model lesson examples on:**
- BurntSushi/ripgrep — https://github.com/BurntSushi/ripgrep
- sharkdp/bat — https://github.com/sharkdp/bat
- sharkdp/fd — https://github.com/sharkdp/fd
- clap-rs/clap — https://github.com/clap-rs/clap
- tokio-rs/mini-redis — https://github.com/tokio-rs/mini-redis

**For algorithms in readable form (cite in lessons):**
- EbTech/rust-algorithms — https://github.com/EbTech/rust-algorithms — *Whitebox cookbook. When a lesson needs a concrete algorithm example, prefer EbTech's version over petgraph's for readability.*

**For the eframe/egui app shell (CORE DEPENDENCY — emilk):**
- emilk/egui — https://github.com/emilk/egui — *Primary UI dependency. Read `egui_demo_app` for best-practice patterns. Study the immediate-mode state model before writing any UI code.*
- emilk/eframe_template — https://github.com/emilk/eframe_template — *Use as the bootstrap template for `rusty-app`. Strip the WASM scaffolding; we don't need it.*

**For the embedded terminal (Phase 1 — high-risk):**
- wez/wezterm — https://github.com/wezterm/wezterm — *Pure-Rust terminal emulator. Reference implementation for PTY + ANSI rendering. Not a direct dependency; a model.*
- alacritty/alacritty — https://github.com/alacritty/alacritty — *Same idea, OpenGL-based. Reference only.*
- alacritty/vte — https://github.com/alacritty/vte — **direct dependency for ANSI escape parsing.**
- wezterm/wezterm/pty — packaged as `portable-pty` on crates.io — **direct dependency for cross-platform PTY.**
- a-b-street/egui_term — search latest; evaluate maturity in Phase 0.

**For parsing learner code (grader AST hints):**
- dtolnay/syn — https://github.com/dtolnay/syn
- oli-obk/cargo_metadata — https://github.com/oli-obk/cargo_metadata — **direct dependency for `cargo --message-format=json` parsing.**

**For the LSP integration (Phase 4):**
- rust-lang/rust-analyzer — https://github.com/rust-lang/rust-analyzer — runtime dependency (subprocess), not a Rust dep
- ebkalderon/tower-lsp — https://github.com/ebkalderon/tower-lsp — *LSP client patterns; we're a client not a server but the protocol structures are the same*
- gluon-lang/lsp-types — https://github.com/gluon-lang/lsp-types — **direct dependency for typed LSP messages.**

**For error handling:**
- dtolnay/anyhow — https://github.com/dtolnay/anyhow
- dtolnay/thiserror — https://github.com/dtolnay/thiserror

**For lesson content loading:**
- toml-rs/toml — frontmatter parsing
- raphlinus/pulldown-cmark — Markdown rendering for lesson prose

**For graph / data structures in lesson examples:**
- petgraph/petgraph — https://github.com/petgraph/petgraph
- crossbeam-rs/crossbeam — https://github.com/crossbeam-rs/crossbeam
- rayon-rs/rayon — https://github.com/rayon-rs/rayon

**Curated meta-lists:**
- rust-unofficial/awesome-rust — https://github.com/rust-unofficial/awesome-rust

### Pedagogy research (cite in `docs/PEDAGOGY.md`)

- Roediger, H. L., & Karpicke, J. D. (2006). Test-enhanced learning. *Psychological Science*, 17(3), 249–255.
- Cepeda, N. J., Pashler, H., Vul, E., Wixted, J. T., & Rohrer, D. (2006). Distributed practice in verbal recall tasks. *Psychological Bulletin*, 132(3), 354–380.
- Sweller, J. (1988). Cognitive load during problem solving. *Cognitive Science*, 12(2), 257–285.
- Renkl, A., & Atkinson, R. K. (2003). Structuring the transition from example study to problem solving. *Educational Psychologist*, 38(1), 15–22.
- Kapur, M. (2008). Productive failure. *Cognition and Instruction*, 26(3), 379–424.
- Shute, V. J. (2008). Focus on formative feedback. *Review of Educational Research*, 78(1), 153–189.
- Bjork, R. A., & Bjork, E. L. (1992). A new theory of disuse and an old theory of stimulus fluctuation. *From Learning Processes to Cognitive Processes*. (For "desirable difficulties.")
- Wozniak, P. A. (1990). *Optimization of learning.* Master's thesis. (Origin of SM-2.)
- Brown, J. S., Collins, A., & Duguid, P. (1989). Situated cognition and the culture of learning. *Educational Researcher*, 18(1), 32–42.
- Lave, J., & Wenger, E. (1991). *Situated Learning: Legitimate Peripheral Participation.* Cambridge University Press.

---

## 8. Acceptance Criteria

MVP is "done" when all of these hold:

1. `git clone` + `./install.sh` on a clean macOS, Linux, and Windows machine completes successfully and launches Rusty.
2. The install script detects missing Rust, asks consent, and runs rustup if confirmed. Declining exits cleanly.
3. Rusty's main window opens with a working embedded terminal that can run `cargo --version` and render ANSI color.
4. The terminal's `cwd` is locked to the active lesson's sandbox. `cd /` is intercepted and refused.
5. All eight Foundations lessons render and are walkable end-to-end.
6. Each lesson's three exercises (Worked + Faded + Open) are completable, with `cargo test`-based grading rendering real `rustc` diagnostics.
7. rust-analyzer integration: editing a file shows squiggles for real errors within < 2 seconds.
8. Spaced repetition reviews appear after their intervals (test by clock manipulation).
9. Progress survives app restart.
10. Keyboard-only completion of any lesson is possible.
11. `docs/PEDAGOGY.md` cites every research claim with a real reference.
12. `docs/CONTENT_AUTHORING.md` enables a non-Rust author to add a ninth lesson by creating one directory.
13. GitHub Actions builds + tests on all three OSes on push to `main`.
14. `cargo test` passes across all crates. Grader has unit tests for every `SuccessCriterion` variant. Curriculum loader has property tests.

---

## 9. Working Style for Claude Code

Maps to user preferences and Sprint Loops methodology.

1. **Read this prompt entirely before any code.**
2. **Surface clarifying questions in ONE batch up front.** Don't drip them. User is time-constrained.
3. **Confirm Phase 0 decisions explicitly** before Phase 1 begins. Especially PTY library and ANSI renderer.
4. **Phase boundaries are commit points.** Don't advance until phase N runs end-to-end.
5. **Use real crate versions, not placeholders.** Check `crates.io` for current versions before adding to `Cargo.toml`.
6. **No `todo!()` or `unimplemented!()` in committed code** unless explicitly deferred. If you want to stub, stop and ask.
7. **Run the code.** Don't claim phase done without launching the binary and clicking through.
8. **Show compiler errors verbatim** when you hit them. User reads `rustc` output fluently and wants the real text.
9. **When choosing between two designs, present both with tradeoffs, recommend one, wait.**
10. **No flattery.** No "great question!" Get to the answer.
11. **`[[wikilinks]]` in any markdown deliverable.** Wrap project names (Rusty, Tescellate, Carbide), framework names (egui, eframe, cargo, rust-analyzer), and conceptual terms (Ownership, Borrow Checker, Spaced Repetition, PTY, LSP) in double-brackets. User's vault needs the link graph.
12. **TOML or YAML frontmatter on every `.md` file you create.** Include `tags`, `project`, `related`, `status`.
13. **User learns by typing commands manually.** In Phase 0–1, show commands and wait for user to run them. From Phase 2 onward you may execute, but show the command first.

---

## 10. First-Turn Checklist

Your first response must be exactly these three things in this order:

1. **Restate the goal in three sentences.** Confirm understanding.
2. **List your clarifying questions** as a batched, numbered list. Aim for under 10. Include the PTY library choice and ANSI renderer decision as questions 1 and 2.
3. **State what you propose to do first** (which should be "create the workspace skeleton" after questions are answered).

**Do not write any Rust code in the first turn. Do not create any files in the first turn. Wait for the user's answers.**

---

## 11. Tripwires (stop and reread this prompt if any of these happen)

- Authoring lessons in HTML, JS, or anything non-Rust.
- Reaching for a JavaScript dependency because "it's easier in JS."
- Adding a backend server.
- Trying to compile Rust in the browser.
- Bundling `rustc` with Rusty (we depend on the system toolchain).
- Skipping the PTY spike to "get to the fun stuff."
- Skipping a phase's commit point because "the next phase is small."
- Stubbing the grader because "we'll grade properly later."
- Writing pedagogy claims without a citation in `docs/PEDAGOGY.md`.
- Letting native-only code (`std::process`, raw filesystem outside sandbox) leak into `rusty-curriculum`, `rusty-scheduler`, `rusty-grader`, or `rusty-pedagogy`. Those four stay portable; everything OS-dependent goes in `rusty-host`.
- Allowing the embedded terminal to escape the sandbox.
- Saying "great idea!" instead of just answering.

---

## 12. Mascot and Voice

**Rusty is named after the user's dog: large, orange, presumably good.** This is not decoration — it's a constraint on voice and UX.

A large orange dog as the app's mascot implies a specific tone:

- **Patient, not condescending.** A good dog does not sigh when you repeat the recall question wrong; he waits.
- **Enthusiastic about small wins.** Passing your first `cargo check` should feel like coming home with groceries — disproportionately celebrated.
- **Never judgmental about mistakes.** The borrow checker is already strict enough. Rusty (the app) is the part that's on the learner's side.
- **Physically present in the UI, but never in the way.** A small idle illustration in a corner, an ear-perk on a correct answer, a head-tilt on an error. No barking pop-ups. No "Rusty wants to teach you about Lifetimes!" modals. Think Clippy's *opposite*.

**Mascot deliverables for Phase 7 (polish):**
- A single SVG illustration of Rusty in three states: idle (sitting), happy (tail up, ears forward), thinking (head tilt). Hand-drawn or commissioned; do not generate with AI image tools — the user has a real dog and the mascot should look like *a* dog, not the median of all dogs.
- A 16×16 favicon derived from the idle pose (for the README and the GitHub repo).
- No animation in MVP. Static state transitions only. Animation is Phase 8+.

**Naming hygiene throughout the codebase:**
- Crate prefix is `rusty-` (already in the layout).
- Application title bar reads "Rusty — Learn Rust by Doing." Subtitle optional.
- Error messages and UI copy refer to the app as "Rusty," not "the app" or "this tool." First person ("I noticed you...") is forbidden — first-person mascots are uncanny. Third-person observational ("Looks like the borrow checker isn't happy with line 4") is the voice.
- Do *not* anthropomorphize Rust the language as Rusty the dog. They are different entities. Rusty teaches Rust; Rusty is not Rust.

### The architectural lesson hiding in here

The mascot constraint maps to a real architecture principle: **the app's personality lives in one file, not scattered.** Create `crates/rusty-app/src/voice.rs` containing every user-facing string and every mascot-state trigger. When the user wants to change tone, translate, or A/B test encouragement levels, it's one file. Push variability to the edges; keep the engine fixed. The `rusty-app` binary should be feature-complete after Phase 7 and never need to change to ship new content — if a Phase 8 lesson requires app-code changes, the curriculum model in Section 3 has a missing degree of freedom; patch the model, don't special-case the lesson.

If you find yourself writing UI copy inline in an egui widget, stop and move it to `voice.rs`. Same for mascot state changes — they should be triggered by events emitted from the grader and scheduler, not by `if` statements in the renderer.

---

*End of prompt. Begin by executing Section 10.*
