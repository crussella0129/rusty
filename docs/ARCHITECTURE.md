# Rusty — Architecture Spec

This document details the internal design of [[Rusty]], covering the subprocess model, the mascot state machine, and the keyboard navigation architecture.

---

## 1. Subprocess Architecture

Rusty is a native desktop application whose core executor relies on three subprocess relationships managed by `rusty-host`:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Rusty Native App (egui)                        │
│                                                                        │
│  ┌─────────────────────────┐  ┌─────────────────────────────────────┐  │
│  │   Lesson Pane           │  │   Workspace Pane                    │  │
│  │  - prose                │  │  ┌────────────────────────────────┐ │  │
│  │  - recall reviews       │  │  │ Code Editor                    │ │  │
│  │  - Mascot companion     │  │  │ - Renders files                │ │  │
│  │                         │  │  ├────────────────────────────────┤ │  │
│  │                         │  │  │ Embedded Terminal              │ │  │
│  │                         │  │  │ - PTY-attached sandbox shell   │ │  │
│  │                         │  │  └────────────────────────────────┘ │  │
│  └─────────────────────────┘  └─────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
          │                         │                        │
          ▼ (PTY spawn)             ▼ (cargo-test spawn)     ▼ (JSON-RPC)
  1. Sandboxed Shell       2. Grading Grader        3. rust-analyzer LSP
```

### 1.1 Sandboxed Shell (PTY)
Spawns a persistent, interactive terminal process using the `portable-pty` crate. The terminal locks the user's `cwd` into `<rusty-repo>/workspace/lessons/<lesson-id>/`. 
- **Enforcement**: Any `cd` command trying to navigate out of the sandbox root is intercepted at the keystroke level and cancelled via Ctrl-C (`0x03`), writing `CD_REFUSED` back to the terminal performer.

### 1.2 Grading Process (Process #2)
Spawns `cargo test --message-format=json` dynamically inside the lesson sandbox when the user clicks "Check". The grader parses compiler errors and test results without freezing the UI.
- **Enforcement**: Manifest paths are locked using `--manifest-path` to prevent ancestor workspace compilation leaks.

### 1.3 Language Server Protocol (LSP)
Spawns a persistent `rust-analyzer` instance in the sandbox folder. Communicates over stdin/stdout using JSON-RPC, feeding live gutter diagnostic squiggles, type hovers, and completion dropdowns into the editor.

---

## 2. Mascot & State Machine

The app's personality is centralized in `crates/rusty-app/src/voice.rs` (prompt §12). The mascot representation (`MascotState`) maps directly to SVG vector drawings under `assets/` and is managed by a structured event loop.

### 2.1 Mascot State Flow:
- **Idle (sitting)**: Default pose showing the mascot waiting patiently.
- **Thinking (head tilt)**: Triggered by grading starts (`handle_grade_start`) or when checking yields warnings or errors (`handle_verdict`).
- **Happy (tail up, ears perked)**: Triggered on successful checks or successfully answered spaced-repetition recall reviews.

Grading and scheduler components emit events to the `Mascot` struct. This guarantees the mascot's emotional reactions remain decoupled from raw UI drawing logic.

---

## 3. Keyboard Accessibility

Rusty coordinates pane-to-pane keyboard navigation via global hotkey consumption and programmatic focus requests.

### 3.1 Shortcut Map:
- **`F1` / `Alt+L`**: Focuses the Lesson Pane (routes focus to active check button or recall input field).
- **`F2` / `Alt+E`**: Focuses the Code Editor (requests focus on multiline text buffer).
- **`F3` / `Alt+T`**: Focuses the Terminal (requests focus on terminal responder).

### 3.2 Focus Routing Mechanics:
1. Global hotkeys are consumed mutably at the window level inside the `ui()` loop via `ui.input_mut()`, preventing key leaks to terminal or editor buffers.
2. The active `FocusTarget` is stored in the state.
3. In the subsequent draw pass, the targeted widget receives `.request_focus()` programmatically using a stable `Id` (e.g. `editor_text_edit` or `recall_short_answer`).
4. Focused terminal viewports paint a 1.5px custom highlighted border to indicate keyboard focus.
