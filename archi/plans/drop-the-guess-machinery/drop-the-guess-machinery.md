# drop-the-guess-machinery

A link had two standings. The second one, evidence, existed for one producer: capture
minting from a shared word. That producer is gone. What is left grades nothing, scores
nothing and decays nothing, and it still shows in the help — a verb with no live input,
which answers the reader with silence instead of an error.

Six things leave together: the standing itself, `link confirm`, the `--evidence` filter, the
confidence score, its erosion and the decay event, the `decayed_evidence` finding and
`audit --prune`. The rows stay. A journal an older binary wrote still folds.

## Stack

- Rust — existing codebase, `crates/archi`
- cargo test, plain `#[test]` files — existing convention, user choice on earlier plans, no
  new dependency
- `crates/archi/tests/link_e2e.rs` — existing home for the link surface end to end
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Links` — the code-link surface: the verbs, the journal, the grader and capture
- `Links` realizes `crates/archi/src/links/` and its arm of `crates/archi/src/main.rs`
