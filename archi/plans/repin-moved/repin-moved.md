# repin-moved

A crate rename orphans every link into it at once. The grader proves each move by body
hash and prints the exact new address; the only consumer is a person typing repin one row
at a time, and on a whole rename the session breaks. One verb accepts the exact candidates
in bulk. Judgement stays per-row: inexact candidates are reported, never taken.

## Stack

- Rust — existing codebase, `crates/archi`
- cargo test, plain `#[test]` files — existing convention
- `crates/archi/tests/link_e2e.rs` — existing home of the link-surface tests
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Links` — the code-link surface: the verbs, the journal, the grader
- `Links` realizes `crates/archi/src/links/` and its arm of `crates/archi/src/main.rs`
