# the-file-is-the-unit

The wave gate chose what to demand by comparing words: the words of a changed file against
the words of a model name. It demanded elements a change never touched and stayed silent on
the ones it did. The delta replaces the comparison, and the file replaces the spec ref.

## Stack

- Rust — existing codebase, `crates/archi`
- cargo test, plain `#[test]` files — existing convention, user choice on earlier plans, no
  new dependency
- `crates/archi/tests/plan_e2e.rs` — existing home: it drives the wave loop to its gate
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Planner` — the wave lifecycle and the gates that close a wave
- `Planner` realizes `crates/archi/src/plans/mod.rs` and `crates/archi/src/links/capture.rs`
