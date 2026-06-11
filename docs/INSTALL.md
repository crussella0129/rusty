# Rusty — Installation Guide

[[Rusty]] is distributed GitHub-native — the repository *is* the product. This document outlines setup procedures, system requirements, and troubleshooting steps for compiling and running the application.

---

## System Requirements

To build and run Rusty, your environment must meet the following baseline:
- **Operating System**: macOS (Intel/Apple Silicon), Linux (x86_64, with standard X11/Wayland libs), or Windows 10/11 (x86_64).
- **Git**: Installed and configured on your terminal.
- **C/C++ Build Tools**: 
  - On macOS: Xcode Command Line Tools (`xcode-select --install`).
  - On Linux: `build-essential` or equivalent package.
  - On Windows: MSVC C++ Build Tools (installed automatically via Visual Studio Installer or build tools installer).

---

## Interactive Bootstrap Setup

Rusty provides automated installation scripts at the workspace root to check for the Rust toolchain, build the project, and launch:

```bash
git clone https://github.com/crussella0129/rusty.git
cd rusty

# On Unix/macOS:
./install.sh

# On Windows (PowerShell):
.\install.ps1
```

### Setup Phases:
1. **Toolchain Check**: The script checks if `cargo` and `rustc` are available on your `PATH`.
2. **Consent-Driven Installation**: If Rust is missing, the script will request interactive consent (`y/N`) to install the compiler toolchain via the official rustup installer. It will **never** perform a silent install.
3. **Pedagogical Compilation**: Once a toolchain is confirmed, Cargo compiles all workspace crates in release mode (`cargo build --release`). Compiling Rusty is the learner's first contact with the compiler.
4. **Binary Launch**: Upon successful compilation, the script launches the compiled binary directly.

---

## Troubleshooting

### 'cargo' is not recognized after installation
On some systems, the current shell session does not automatically pick up the new PATH variables added by rustup.
- **Fix**: Restart your terminal window, navigate back to the `rusty` directory, and run the script again.

### Linux build errors (missing libraries)
Egui/Eframe requires standard graphics rendering libraries. If your build fails:
- **Fix (Ubuntu/Debian)**: Run `sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`.
- **Fix (Fedora)**: Run `sudo dnf install clang libxcb-devel libxkbcommon-devel openssl-devel`.
