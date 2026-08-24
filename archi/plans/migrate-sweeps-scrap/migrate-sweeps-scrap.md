# migrate-sweeps-scrap

The close cleans its wave files only from the binary that learned to. A standing project
migrated to today's tool carries `waves/` folders under plans an older binary closed — on
this very tree that was 13M across 69 dead indexes before a hand sweep took them. The
migrate page gains the third measurement and the pass, so the hand sweep never happens
again anywhere.

## Stack

- Rust — existing codebase, `crates/archi`
- the skill sources under `skills/` — `scaffold.rs` embeds them with `include_str!`
- cargo test, plain `#[test]` files — existing convention
- `crates/archi/tests/init_e2e.rs` — existing home of the skill guards
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Scaffold` — embeds the briefing at build time and installs it beside a tree
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the skill sources under `skills/`
