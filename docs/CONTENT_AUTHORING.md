---
project: Rusty
tags: [rusty, content-authoring, lessons, toml, stub]
related:
  - "[[Rusty]]"
  - "[[Curriculum Model]]"
status: stub
---

# Rusty — Content Authoring

> Stub. Filled out in [[Phase 7]]. The acceptance bar (§8.12): a non-Rust author
> can add a ninth lesson by creating one directory under `content/lessons/`, with
> no app-code changes.

A lesson is a directory under `content/lessons/<lesson-id>/` containing a
`lesson.toml` (metadata, prose, recall prompt, exercises), a `starter/` cargo
project, and a `solution/` cargo project. The typed [[Curriculum Model]] in
`rusty-curriculum` (landing in [[Phase 2]]) defines the schema. This doc will walk
through authoring one end-to-end.
