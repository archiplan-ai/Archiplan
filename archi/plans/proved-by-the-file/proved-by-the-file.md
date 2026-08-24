# proved-by-the-file

--proved-by demands a test symbol everywhere, written on the Rust case. Every other
language canonicalizes whole-file by construction — no symbol can ever resolve — so
declarations in a TypeScript project are a dead end: the symbol is unparseable and the
bare file is refused. The demand learns to follow the canonicalizer.

## Stack

- Rust — existing codebase, `crates/archi`
- cargo test, plain `#[test]` files — existing convention
- `crates/archi/tests/link_e2e.rs` — existing home of the declaration tests
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Links` — the code-link surface: the declaration verb, capture, the journal
- `Links` realizes `crates/archi/src/links/` and its arm of `crates/archi/src/main.rs`
