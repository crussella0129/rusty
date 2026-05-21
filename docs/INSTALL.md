---
project: Rusty
tags: [rusty, install, bootstrap, rustup, stub]
related:
  - "[[Rusty]]"
  - "[[Cargo]]"
status: stub
---

# Rusty — Install

> Stub. Filled out in [[Phase 7]] (bootstrap + ship), when `install.sh` /
> `install.ps1` exist.

[[Rusty]] is distributed GitHub-native — the repository *is* the product:

```bash
git clone https://github.com/crussella0129/rusty.git
cd rusty
./install.sh        # or .\install.ps1 on Windows
```

The bootstrap detects the Rust toolchain (offering to run rustup with consent if
absent), builds Rusty from source via `cargo build --release` (the first compile is
itself the first lesson), and launches `./target/release/rusty`. Updating is
`git pull && cargo build --release`. See `RUSTY_PROMPT.md` §0 for the full contract.
