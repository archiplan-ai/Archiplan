# archi-resume

A fresh session's first question is not "how do I enter a seat" — every workflow skill
already opens with that — but "which seats stand, in what state, and which skill continues
each". Nothing answers it, so sessions improvise: git archaeology first, reports about the
wrong round, a harness boundary misread as a wall. One page answers it and routes.

## Stack

- Rust — existing codebase, `crates/archi`
- the skill sources under `skills/` — `scaffold.rs` embeds them with `include_str!`
- cargo test, plain `#[test]` files — existing convention
- `crates/archi/tests/init_e2e.rs` — existing home of the skill guards
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Scaffold` — embeds the briefing at build time and installs it beside a tree
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the skill sources under `skills/`
