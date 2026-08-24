# declaration-over-search

Stop inferring which spec a piece of code answers and have the writer say it. A task's
sub-agent leaves one declaration file beside the wave index; `plan next` reads it instead of
matching shared words, refuses while a changed symbol is named nowhere, and mints the
declared pairs asserted. A requirement gains an address so half the declarations have
somewhere to go, and every row records which rule produced it, so two thousand inferred
candidates stay readable as history and stop being read as evidence.

## Stack

- Rust — existing codebase, `crates/archi` and `crates/modeling-lang`
- TOML for the declaration file — user choice: line-oriented, so a model rarely breaks it and every parse error carries a line; `archi.toml` and `worktrees.toml` already stand in the tree
- toml crate — already a dependency, and its errors carry span information the refusal needs
- cargo test, plain `#[test]` integration files — existing convention, no dev-dependencies
- `crates/archi/tests/link_e2e.rs` and `crates/archi/tests/plan_e2e.rs` — existing files, no new suite
- no runtime infrastructure — archi is a CLI over files and git and never reaches the network
- the reverse view rides `link ls --spec` and `link audit` — spec decision: the read surface renders from the journal and no verb writes into a spec file

## Architecture

- `Links` — the spec-ref shapes, the journal rows and their grading, all in one file
- `Links` realizes `crates/archi/src/links/mod.rs`
- `Planner` — the wave gate: what a task owes before the next wave opens
- `Planner` realizes `crates/archi/src/plans/mod.rs`
