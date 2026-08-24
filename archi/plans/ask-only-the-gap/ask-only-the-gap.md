# ask-only-the-gap

The migration interview asks four questions per candidate while the candidate's own prose
answers three of them, and the explain port dropped three pieces of the old discipline.
Both pages learn the same rule: what the record already answers is not asked again — and
the one answer that must never come from paper stays asked.

## Stack

- Rust — existing codebase, `crates/archi`
- the skill sources under `skills/` — `scaffold.rs` embeds them with `include_str!`
- cargo test, plain `#[test]` files — existing convention
- `crates/archi/tests/init_e2e.rs` — existing home of the skill guards
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Scaffold` — embeds the briefing at build time and installs it beside a tree
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the skill sources under `skills/`
