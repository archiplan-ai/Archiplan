# archi-search

The retrieval order lives in one skill's ground rule, and the six other skills that
search never see it. Each agent then improvises its own way in. One page carries the
doctrine, every working skill points at it by name, and guards hold both halves: the
pointer everywhere, the vocabulary in one place.

## Stack

- Rust — existing codebase, `crates/archi`
- the skill sources under `skills/` — `scaffold.rs` embeds them with `include_str!`
- cargo test, plain `#[test]` files — existing convention
- `crates/archi/tests/init_e2e.rs` — existing home of the skill guards
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Scaffold` — embeds the briefing at build time and installs it beside a tree
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the skill sources under `skills/`
