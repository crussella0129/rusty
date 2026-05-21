---
project: Rusty
owner: Charles Russella
created: 2026-05-21
updated: 2026-05-21
tags: [rust, wasm, egui, pedagogy, claude-code, prompt, learning, tescellate-adjacent]
related:
  - "[[Tescellate]]"
  - "[[Carbide]]"
  - "[[Rust]]"
  - "[[Sprint Loops]]"
  - "[[Claude Thoughts Vault]]"
status: draft-spec
sprint_phase: 0-spec
---

# Rusty — Claude Code Build Prompt

**A 100% Rust + WebAssembly + egui tutorial application that teaches Rust through research-backed pedagogy and in-app interactive exercises.**

> Paste this entire file into Claude Code as the initial prompt. Claude Code should read it, ask any clarifying questions in a single batch, then execute the phased plan. Do **not** skip phases. Do **not** stub things out "for later" unless explicitly noted as a Phase N+1 deferral.

---

## 0. Identity and Constraints

### Project name
**Rusty** — named after the user's dog, who is large and orange. See Section 12 for the full mascot/voice spec; this is not optional flavor, it constrains the UI.

### Hard technical constraints

These are non-negotiable. Flag any tension before resolving.

1. **100% Rust.** No JavaScript source files. WASM glue from `wasm-bindgen` is acceptable because it is generated, not authored.
2. **`egui` for UI.** Use `eframe` as the application shell. The same binary targets native (`cargo run`) and WASM (`trunk serve`).
3. **WASM target is the primary deployment.** Native build exists for fast dev iteration and is a non-negotiable second-class target.
4. **In-app exercise execution.** Exercises must compile and run *inside the browser*, not on a server. See Section 5 for the execution model.
5. **No backend server required for the MVP.** Static hosting only (GitHub Pages, Cloudflare Pages, or local file).
6. **Persistence via browser `localStorage`** (through `eframe`'s built-in persistence layer). No cloud sync in MVP.
7. **Accessibility:** keyboard-only navigation must work for every exercise. Screen-reader labels on all interactive elements.
8. **MSRV** (Minimum Supported Rust Version): latest stable at build time. Document the exact version in `rust-toolchain.toml`.

### Stretch constraints (flag tradeoffs, don't silently violate)

- Bundle size target: **< 5 MB** gzipped WASM for the MVP. Use `wasm-opt -Oz`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`.
- First-paint target: **< 2 seconds** on a 10 Mbps connection.
- Cold start (first interactive): **< 5 seconds**.

### What we are NOT building in MVP

- No multi-user accounts.
- No social features (leaderboards, sharing).
- No payment / monetization.
- No mobile-specific UI (responsive web is fine; native mobile app is out).
- No AI tutor / LLM integration. (Phase 4+ consideration only.)
- No content beyond the lessons specified in Section 4. We are building the *system*, with a representative slice of content to validate it.

---

## 1. Why This Exists (Pedagogical Thesis)

Rusty is not "the Rust Book in egui." It is an attempt to operationalize four research-backed learning principles that most programming tutorials violate:

### 1.1 Retrieval practice over re-reading
**Source:** Roediger & Karpicke (2006), *Test-Enhanced Learning: Taking Memory Tests Improves Long-Term Retention*. Psychological Science, 17(3).

**Finding:** Active recall (being tested) produces dramatically better long-term retention than repeated study, even when learners *believe* re-reading is more effective.

**Implication for Rusty:** Every lesson ends with a recall prompt *before* any exercise. The recall is short-answer or multiple-choice on the lesson's key claim, graded immediately. No skipping.

### 1.2 Spaced repetition
**Source:** Cepeda, Pashler, Vul, Wixted, & Rohrer (2006), *Distributed Practice in Verbal Recall Tasks: A Review and Quantitative Synthesis*. Psychological Bulletin, 132(3).

**Finding:** Spacing reviews over expanding intervals produces 2–3× better retention than massed practice, with an optimal review gap roughly 10–30% of the desired retention interval.

**Implication for Rusty:** Use a simplified SM-2 scheduler (the Anki algorithm) for concept reviews. Each lesson has 3–5 "atomic concepts" tracked individually. Each session opens with due reviews before new material.

### 1.3 Worked examples → faded scaffolding → independent practice
**Source:** Sweller (1988), *Cognitive Load During Problem Solving*. Cognitive Science, 12. And: Renkl & Atkinson (2003) on the *expertise reversal effect* and faded examples.

**Finding:** Novices learn far more from studying worked examples than from solving problems unaided. The benefit reverses as expertise grows — experts learn more from problem-solving. The bridge is *faded* examples: start fully worked, progressively blank out steps.

**Implication for Rusty:** Every exercise type comes in a triple — `Worked` (read and annotate), `Faded` (fill in 1–3 blanks), `Open` (write from scratch). Learner advancement through the triple is driven by performance, not seat time.

### 1.4 Productive failure with immediate compiler feedback
**Source:** Kapur (2008), *Productive Failure*. Cognition and Instruction, 26(3). Combined with the established literature on immediate vs. delayed feedback (Shute, 2008).

**Finding:** Struggling productively *before* receiving instruction improves transfer. But once struggling, feedback should be immediate and specific.

**Implication for Rusty:** Some exercises present the problem *before* the lesson explanation (productive failure mode), then the lesson reveals what the learner was trying to discover. The Rust compiler's error messages are leveraged as the immediate-feedback channel — Rusty surfaces the real `rustc`/`miri` output, annotated.

### 1.5 The Rust-specific bet

Rust's compiler is the best feedback teacher in mainstream languages. Most Rust tutorials waste this by paraphrasing errors instead of showing them. Rusty does the opposite: **the compiler's actual output is the primary teaching surface.** Lessons are scaffolding around compiler conversations.

---

## 2. Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                     Rusty (eframe app)                      │
│                                                                │
│  ┌────────────┐  ┌────────────┐  ┌─────────────────────────┐ │
│  │  Lesson    │  │  Exercise  │  │  Scheduler (SM-2-lite)  │ │
│  │  Renderer  │  │  Runner    │  │  + Progress Store       │ │
│  └─────┬──────┘  └──────┬─────┘  └──────────┬──────────────┘ │
│        │                │                    │                │
│  ┌─────▼────────────────▼────────────────────▼─────────────┐ │
│  │              Curriculum Model (typed)                    │ │
│  │  Lesson → Concept[] → Exercise[] (Worked|Faded|Open)     │ │
│  └──────────────────────────┬───────────────────────────────┘ │
│                             │                                  │
│  ┌──────────────────────────▼───────────────────────────────┐ │
│  │           In-Browser Rust Execution Layer                │ │
│  │  - Static analyzer (syn-based) for shape checks          │ │
│  │  - rustc_codegen_cranelift-via-wasm OR remote eval (TBD) │ │
│  │  - Sandboxed runner with stdout/stderr capture           │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │  Persistence (eframe::Storage → localStorage on WASM)    │ │
│  └──────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### The "in-browser Rust execution" question

This is the **single hardest architectural decision** and you must surface it in your first clarifying-questions batch. Three options, in increasing order of ambition:

**Option A — Pattern-match grader (lowest risk).** Use `syn` (compiled to WASM) to parse the learner's submitted code into an AST and check structural properties: "did they call `iter().map()`?", "is this function pure?", "does this match expected signature?". Compilation and execution happen *conceptually* but not literally. Exercises are designed to be graded by AST shape and unit-test-like assertions over expected output strings the learner provides.

  - **Pro:** Trivially achievable. `syn` works in WASM today. Ships in MVP.
  - **Con:** Doesn't actually run the code. Learners can't see runtime output of their own programs. Borrow-checker errors can't be shown for arbitrary code.

**Option B — Hosted compile endpoint (medium risk, breaks "no backend" rule).** Send code to a sandboxed `rustc` instance. The Rust Playground does this. We could embed a thin client.

  - **Pro:** Full real-rustc errors. Real execution.
  - **Con:** Violates Section 0 constraint #5 (no backend). Defer to Phase 3+.

**Option C — `rustc` in WASM (highest risk, true to spec).** Compile `rustc` itself to WASM, or use `rustc_codegen_cranelift` + a WASM-hosted Cranelift to compile learner code to WASM in the browser, then execute it in a nested WASM sandbox.

  - **Pro:** True 100% Rust + WASM. No backend. Real compiler.
  - **Con:** As of mid-2026, this is research-tier. `rustc` is a multi-hundred-MB compiler. Cranelift-as-WASM exists in experimental form but the toolchain is fragile. Bundle size will blow past the 5 MB target unless we ship `rustc` as a separately-loaded chunk.

**Default recommendation for MVP:** Option A, with the architecture designed so Option C can replace the executor module without touching the rest of the app. The `Executor` trait abstracts this:

```rust
pub trait Executor: Send + Sync {
    fn analyze(&self, code: &str, exercise: &Exercise) -> AnalysisResult;
    fn execute(&self, code: &str, stdin: &str) -> ExecutionResult;
}

pub struct SynPatternExecutor;       // Phase 1 default
pub struct PlaygroundExecutor;       // Phase 3 optional
pub struct CraneliftWasmExecutor;    // Phase 4+ research
```

**Confirm this decision with the user before writing any executor code.**

### Core crate layout

```
rusty/
├── Cargo.toml                  # workspace
├── rust-toolchain.toml
├── crates/
│   ├── rusty-app/            # eframe binary, native + wasm targets
│   ├── rusty-curriculum/     # typed lesson/exercise model + content loader
│   ├── rusty-scheduler/      # SM-2-lite spaced repetition
│   ├── rusty-executor/       # trait + SynPatternExecutor impl
│   ├── rusty-grader/         # exercise grading rules
│   └── rusty-pedagogy/       # research citations + helpers, kept honest
├── content/
│   └── lessons/                # .toml-fronted .md lesson files
├── assets/                     # fonts, icons (egui-loadable)
├── web/
│   ├── index.html              # trunk template
│   └── style.css               # minimal, egui handles most
└── docs/
    ├── ARCHITECTURE.md
    ├── PEDAGOGY.md             # the research, citations, application
    └── CONTENT_AUTHORING.md
```

---

## 3. Curriculum Data Model

The model is typed Rust, not free-form Markdown. Content lives in TOML-fronted Markdown files but is deserialized into these types at load time.

```rust
pub struct Lesson {
    pub id: LessonId,                    // e.g. "ownership-01-moves"
    pub title: String,
    pub track: Track,                    // Foundations, Intermediate, etc.
    pub prereqs: Vec<LessonId>,
    pub estimated_minutes: u8,
    pub concepts: Vec<Concept>,          // 3–5 atomic concepts
    pub body: Vec<Block>,                // prose, code, callouts
    pub recall_prompt: RecallPrompt,     // mandatory before exercises
    pub exercises: Vec<Exercise>,
    pub further_reading: Vec<Reference>, // from the resources list
}

pub struct Concept {
    pub id: ConceptId,                   // tracked individually for SRS
    pub claim: String,                   // one-sentence statement
    pub why_it_matters: String,
    pub common_misconception: Option<String>,
}

pub enum Exercise {
    Worked(WorkedExample),               // annotated read-through
    Faded(FadedExample),                 // fill blanks
    Open(OpenChallenge),                 // write from scratch
    PredictThenRun(PredictExercise),     // productive failure
}

pub struct PredictExercise {
    pub code: String,
    pub question: String,                // "What does this print?"
    pub correct_answer: AnswerSpec,
    pub reveal_after: RevealMode,        // immediate | after-attempt
    pub explanation: String,             // shown after answer
}
```

The `Exercise` enum is the heart of the pedagogical model. Each lesson must contain at least one exercise of *each* variant to enforce the Worked → Faded → Open progression.

---

## 4. MVP Content Scope

Build **one complete track** — *Foundations* — to validate the system. Eight lessons:

1. **Hello, compiler.** What `cargo run` actually does. Reading your first compile error.
2. **Variables, mutability, shadowing.** Why immutable-by-default.
3. **Ownership basics: move semantics.** The single hardest concept; budget two lessons.
4. **Ownership continued: borrows and lifetimes (informal).** No `'a` syntax yet.
5. **Structs and methods.**
6. **Enums and pattern matching.** `Option<T>` introduced here, not earlier.
7. **Error handling with `Result<T, E>` and `?`.**
8. **Collections: `Vec`, `String`, `HashMap`.** And iterators as a teaser for the next track.

Each lesson: ~5 minutes of reading, 3 exercises minimum (one of each Worked/Faded/Open), one PredictThenRun, one recall prompt.

After this track ships, content scales by authoring more TOML files — no code changes.

---

## 5. In-App Execution Model (MVP — Option A details)

The `SynPatternExecutor` works like this:

1. Learner types code into an egui code editor (`egui_extras::syntax_highlighting` for Rust).
2. On "Check" press, code is parsed with `syn::parse_file`.
3. `Grader` runs the exercise's rule set against the AST:
    - **Structural rules:** `must_contain_fn("foo")`, `must_use_iter_adapter("map")`, `must_not_use_clone()`, `must_match_signature(...)`.
    - **Token rules:** `forbidden_tokens(["unsafe"])`, `required_tokens(["match"])`.
    - **Shape rules:** `function_is_pure("foo")`, `no_unwrap_outside_main()`.
4. For `PredictThenRun`, the learner submits the *predicted output*, not code. Grader is a string comparator with whitespace tolerance.
5. Compiler-style feedback: graders return structured `Diagnostic` values rendered in egui to look and feel like `rustc` output (same color scheme, same arrow underlines, same "help:" lines).

The diagnostic renderer is **the most important UX surface in the app.** Get it right early. It is the operationalization of pedagogy thesis 1.5.

---

## 6. Phased Build Plan

Each phase ends with a working, demonstrable app. Don't start the next phase until the previous one runs end-to-end.

### Phase 0 — Spec lock and skeleton (target: 1 day)
- Read this prompt fully. Surface clarifying questions in **one batch**.
- Confirm Executor decision (default to Option A unless user overrides).
- Create the workspace skeleton (all crates, empty libs).
- `cargo check` and `trunk build` both succeed with empty UI ("Rusty" title only).
- Commit: `chore: workspace skeleton`.

### Phase 1 — Curriculum model + one lesson (target: 2–3 days)
- Implement `rusty-curriculum` types and TOML+Markdown loader.
- Author lesson 1 ("Hello, compiler.") as a single content file.
- Implement minimal lesson renderer in egui — just prose and code blocks.
- App displays lesson 1. No exercises yet, no execution.
- Commit: `feat(curriculum): lesson model + lesson 1 renders`.

### Phase 2 — Exercise runner + `SynPatternExecutor` (target: 3–5 days)
- Implement `Executor` trait and `SynPatternExecutor`.
- Implement `Grader` with the rules listed in Section 5.
- Implement the `rustc`-style `Diagnostic` renderer (this is its own little project — give it attention).
- Add three exercises to lesson 1: one Worked, one Faded, one Open.
- All three are completable in-app.
- Commit: `feat(executor): syn-based grading with rustc-style diagnostics`.

### Phase 3 — Recall, scheduling, persistence (target: 3 days)
- Implement `RecallPrompt` (multiple-choice + short-answer).
- Implement `rusty-scheduler` with SM-2-lite. Concepts have `ease`, `interval_days`, `due_at` per learner.
- Implement persistence via `eframe::Storage`. Verify it survives page reload on WASM.
- Add a "Due Reviews" landing screen that appears before new lessons if reviews are due.
- Commit: `feat(scheduler): spaced repetition + persistence`.

### Phase 4 — Content fill: lessons 2–8 (target: 5–7 days)
- Author the remaining seven Foundations lessons in TOML+Markdown.
- For each: 3+ exercises, 1 PredictThenRun, 1 recall prompt, prereqs wired correctly.
- Run through the whole track yourself once before declaring done.
- Commit per lesson: `content(lesson-N): <title>`.

### Phase 5 — Polish and ship (target: 2–3 days)
- Bundle size audit: `wasm-opt -Oz`, measure, optimize.
- Keyboard navigation pass: every interactive element reachable.
- Screen-reader labels via egui's accessibility hooks.
- `docs/CONTENT_AUTHORING.md` so future lessons can be added without touching app code.
- GitHub Actions: build, test, deploy to GitHub Pages.
- README with screenshots and link to live demo.
- Commit: `chore: ship MVP`.

### Phase 6+ — Deferred
- Option B or C executor.
- More tracks (Intermediate, Async, Macros).
- LLM-assisted explanations.
- Community-contributed lessons.

---

## 7. Reference Material

These are the resources Rusty draws from. Use them as primary sources when authoring lesson content and when designing exercises. Lesson `further_reading` fields should cite specific sections of these.

### Canonical Rust resources

**Books and official docs:**

- The Rust Programming Language ("the Book") — https://doc.rust-lang.org/book/
- Brown University interactive Book — https://rust-book.cs.brown.edu/ — *Especially relevant: this is itself a research-backed Rust teaching tool. Study its "ownership inventory" visualizations and quiz patterns; Rusty should be at least as rigorous.*
- Rust by Example — https://doc.rust-lang.org/rust-by-example/
- Rustlings — https://github.com/rust-lang/rustlings — *The closest existing thing to what Rusty is. Study its exercise structure; do not copy it. Rusty's edge is in-app execution and spaced repetition.*
- Programming Rust (Blandy/Orendorff/Tindall, 2nd ed., O'Reilly) — paid, used as a sanity check on technical claims
- Rust for Rustaceans (Gjengset) — intermediate; informs Phase 6+ tracks
- Rust Atomics and Locks (Mara Bos) — https://marabos.nl/atomics/ — for the eventual concurrency track
- The Rust Reference — https://doc.rust-lang.org/reference/
- The Rustonomicon — https://doc.rust-lang.org/nomicon/
- The Cargo Book — https://doc.rust-lang.org/cargo/
- The Little Book of Rust Macros — https://veykril.github.io/tlborm/
- Rust Design Patterns — https://rust-unofficial.github.io/patterns/
- Rust and WebAssembly book — https://rustwasm.github.io/docs/book/ — *Read before Phase 0; informs the trunk+wasm-bindgen setup.*
- `wasm-bindgen` guide — https://rustwasm.github.io/wasm-bindgen/
- Crafting Interpreters (Nystrom) — https://craftinginterpreters.com/ — *Lesson design inspiration; Nystrom's progressive disclosure is the bar.*

**Reading newsletters and blogs:**

- This Week in Rust — https://this-week-in-rust.org/
- Without Boats — https://without.boats/
- fasterthanli.me (Amos) — https://fasterthanli.me/
- Niko Matsakis's blog — https://smallcultfollowing.com/babysteps/

### Repositories to study (and what to take from each)

**For learn-by-doing patterns:**
- rust-lang/rustlings — https://github.com/rust-lang/rustlings — exercise difficulty curve, hint system
- rust-lang/rust-by-example — https://github.com/rust-lang/rust-by-example — minimal-example pedagogy

**For idiomatic Rust to model lesson examples on:**
- BurntSushi/ripgrep — https://github.com/BurntSushi/ripgrep
- sharkdp/bat — https://github.com/sharkdp/bat
- sharkdp/fd — https://github.com/sharkdp/fd
- clap-rs/clap — https://github.com/clap-rs/clap
- tokio-rs/mini-redis — https://github.com/tokio-rs/mini-redis

**For algorithms in readable form (cite in lessons; use as worked-example sources):**
- EbTech/rust-algorithms — https://github.com/EbTech/rust-algorithms — *Whitebox cookbook. When a lesson needs a concrete algorithm example, prefer EbTech's version over petgraph's generic one for readability.*

**For the eframe/egui app shell itself:**
- emilk/egui — https://github.com/emilk/egui — *Primary dependency. Read `egui_demo_app` to see best-practice patterns. Study the immediate-mode state model before writing any UI code.*
- emilk/eframe_template — https://github.com/emilk/eframe_template — *Use as the bootstrap template for the `rusty-app` crate.*

**For parsing learner code (Executor implementation):**
- dtolnay/syn — https://github.com/dtolnay/syn
- chumsky-parser/chumsky — https://github.com/zesterer/chumsky — *only if syn proves insufficient*

**For the diagnostic renderer:**
- rust-lang/annotate-snippets-rs — https://github.com/rust-lang/annotate-snippets-rs — *This is the crate `rustc` itself uses to format diagnostics. Use it directly if it compiles to WASM; otherwise mimic its output format pixel-for-pixel.*

**For error handling:**
- dtolnay/anyhow — https://github.com/dtolnay/anyhow
- dtolnay/thiserror — https://github.com/dtolnay/thiserror

**For incremental computation (Phase 6+; informs grading caching):**
- salsa-rs/salsa — https://github.com/salsa-rs/salsa
- leptos-rs/leptos — https://github.com/leptos-rs/leptos — reactive primitives reference

**For graph and data-structure needs in lessons:**
- petgraph/petgraph — https://github.com/petgraph/petgraph
- crossbeam-rs/crossbeam — https://github.com/crossbeam-rs/crossbeam
- rayon-rs/rayon — https://github.com/rayon-rs/rayon

**Curated meta-lists:**
- rust-unofficial/awesome-rust — https://github.com/rust-unofficial/awesome-rust

### Pedagogy research (cite in `docs/PEDAGOGY.md`)

- Roediger, H. L., & Karpicke, J. D. (2006). Test-enhanced learning: Taking memory tests improves long-term retention. *Psychological Science*, 17(3), 249–255.
- Cepeda, N. J., Pashler, H., Vul, E., Wixted, J. T., & Rohrer, D. (2006). Distributed practice in verbal recall tasks: A review and quantitative synthesis. *Psychological Bulletin*, 132(3), 354–380.
- Sweller, J. (1988). Cognitive load during problem solving: Effects on learning. *Cognitive Science*, 12(2), 257–285.
- Renkl, A., & Atkinson, R. K. (2003). Structuring the transition from example study to problem solving in cognitive skill acquisition: A cognitive load perspective. *Educational Psychologist*, 38(1), 15–22.
- Kapur, M. (2008). Productive failure. *Cognition and Instruction*, 26(3), 379–424.
- Shute, V. J. (2008). Focus on formative feedback. *Review of Educational Research*, 78(1), 153–189.
- Bjork, R. A., & Bjork, E. L. (1992). A new theory of disuse and an old theory of stimulus fluctuation. *From Learning Processes to Cognitive Processes*. (For the "desirable difficulties" framework.)
- Wozniak, P. A. (1990). Optimization of learning. *Master's thesis.* (Origin of SM-2.)

---

## 8. Acceptance Criteria

The MVP is "done" when *all* of the following hold:

1. `cargo run -p rusty-app` opens a native window showing Rusty.
2. `trunk serve` serves the WASM build, which loads in Chrome, Firefox, and Safari with no console errors.
3. Bundle is < 5 MB gzipped.
4. All eight Foundations lessons render with their full content.
5. Every exercise in every lesson is completable in-app, with grading feedback rendered in `rustc`-style.
6. Spaced repetition reviews appear correctly after their intervals (test with system clock manipulation, not real-time waiting).
7. Progress survives page reload.
8. Keyboard-only completion of any lesson is possible.
9. `docs/PEDAGOGY.md` cites every research claim with a real reference. Any claim that can't be cited is flagged or removed.
10. `docs/CONTENT_AUTHORING.md` enables a non-Rust author to add a ninth lesson without touching app code, and the lesson loads correctly.
11. GitHub Actions workflow builds and deploys to GitHub Pages on push to `main`.
12. `cargo test` passes across all crates. Curriculum loader has property tests. Grader has unit tests for every rule type.

---

## 9. Working Style for Claude Code

This is how to execute. These map to the user's working preferences and the Sprint Loops methodology.

1. **Read this prompt entirely before writing any code.** Yes, the whole thing.
2. **Surface clarifying questions in ONE batch up front.** Do not drip them across the session. The user is time-constrained (15-month-old at home).
3. **Confirm the Executor decision explicitly** before any executor code is written. Default to Option A but ask.
4. **Phase boundaries are commit points.** Do not advance to phase N+1 until phase N's deliverable runs end-to-end.
5. **Use real crate versions, not placeholders.** Check `crates.io` for current versions; don't guess.
6. **No `todo!()` or `unimplemented!()` macros in committed code** unless explicitly marked as a deferred phase. If you find yourself wanting to stub, stop and ask.
7. **Run the code.** Don't claim a phase is done without `cargo run` and `trunk serve` both succeeding.
8. **Show compiler errors verbatim** when you hit them; do not paraphrase. The user reads `rustc` output fluently and wants the real text.
9. **When unsure between two designs, present both with tradeoffs**, recommend one, then wait. The user will choose.
10. **No flattery.** No "great question!" No "absolutely!" Get to the answer.
11. **`[[wikilinks]]` in any markdown deliverable** — wrap project names (Rusty, Tescellate, Carbide), framework names (egui, eframe, trunk), and conceptual terms (Ownership, Borrow Checker, Spaced Repetition) in double-brackets. The user files all Claude output into an Obsidian vault and needs the link graph.
12. **TOML or YAML frontmatter on every `.md` file you create.** Include `tags`, `project`, `related` (to other vault notes), and `status`.
13. **The user learns by typing commands manually.** Do not run `cargo init` or `git init` for them in Phase 0. Tell them the exact command and wait. From Phase 1 onward, you may execute, but show the command first.

---

## 10. First-Turn Checklist for Claude Code

When you receive this prompt, your **first response** should be exactly these three things in this order:

1. **Restate the goal in three sentences.** Confirm understanding.
2. **List your clarifying questions** as a batched, numbered list. Aim for under 10. Include the Executor decision (Section 2) as question 1.
3. **State what you propose to do first** (which should be "create the workspace skeleton" once questions are answered).

Do not write any Rust code in the first turn. Do not create any files in the first turn. Wait for the user's answers.

---

## 11. Out-of-Scope Reminders (sanity tripwires)

If you find yourself about to do any of these, stop and re-read this prompt:

- Authoring lessons in HTML, JS, or any non-Rust UI framework.
- Reaching for a JavaScript library because "it's easier in JS."
- Adding a backend "just for the executor."
- Skipping a phase's commit point because "the next phase is small."
- Stubbing the grader because "we'll grade properly later."
- Writing pedagogy claims without a citation in `docs/PEDAGOGY.md`.
- Saying "great idea!" instead of just answering.

---

## 12. Mascot and Voice

**Rusty is named after the user's dog: large, orange, presumably good.** This is not decoration — it is a constraint on voice and UX.

A large orange dog as the app's mascot implies a specific tone:

- **Patient, not condescending.** A good dog does not sigh when you repeat the recall question wrong; he waits.
- **Enthusiastic about small wins.** Passing your first `cargo check` should feel like coming home with groceries — disproportionately celebrated.
- **Never judgmental about mistakes.** The borrow checker is already strict enough. Rusty (the app) is the part that's on the learner's side.
- **Physically present in the UI, but never in the way.** A small idle illustration in a corner, an ear-perk on a correct answer, a head-tilt on an error. No barking pop-ups. No "Rusty wants to teach you about Lifetimes!" modals. Think Clippy's *opposite*.

**Mascot deliverables for Phase 5 (polish):**
- A single SVG illustration of Rusty in three states: idle (sitting), happy (tail up, ears forward), thinking (head tilt). Hand-drawn or commissioned; do not generate with AI image tools — the user has a real dog and the mascot should look like *a* dog, not the median of all dogs.
- A 16×16 favicon derived from the idle pose.
- No animation in MVP. Static state transitions only. Animation is Phase 6+ and requires a real designer.

**Naming hygiene throughout the codebase:**
- The crate prefix is `rusty-` (already in the layout).
- The application title bar reads "Rusty — Learn Rust by Doing." Subtitle is optional and can be cut.
- Error messages and UI copy refer to the app as "Rusty," not "the app" or "this tool." First person ("I noticed you...") is forbidden — it makes mascots creepy. Third-person observational ("Looks like the borrow checker isn't happy with line 4") is the voice.
- Do *not* anthropomorphize Rust the language as Rusty the dog. They are different entities. Rusty teaches Rust; Rusty is not Rust.

### The actual architectural lesson hiding in here

The mascot constraint maps to a real architecture principle: **the app's personality should live in one file, not be scattered.** Create `crates/rusty-app/src/voice.rs` containing every user-facing string and every mascot-state trigger. When the user wants to change tone, translate to another language, or A/B test encouragement levels, it is one file. Push variability to the edges; keep the engine fixed. The `rusty-app` binary should be feature-complete after Phase 5 and never need to change to ship new content — if a Phase 6 lesson requires app-code changes, that's a signal the curriculum model in Section 3 has a missing degree of freedom; patch the model, don't special-case the lesson.

If you find yourself writing UI copy inline in an egui widget, stop and move it to `voice.rs`. Same for mascot state changes — they should be triggered by events emitted from the grader and scheduler, not by `if` statements in the renderer.

---

*End of prompt. Begin by executing Section 10.*
