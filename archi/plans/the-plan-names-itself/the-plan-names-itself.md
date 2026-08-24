# the-plan-names-itself

Step 1 of the planning skill opens every session with a poll about the plan's name, and a
second one on collision. A name is an address, not a decision: the skill derives it and
asks nobody, saving the poll for the choices it genuinely owes the operator.

## Stack

- Rust — existing codebase, `crates/archi`
- the skill sources under `skills/` — `scaffold.rs` embeds them with `include_str!`
- cargo test, plain `#[test]` files — existing convention
- `crates/archi/tests/init_e2e.rs` — existing home of the skill guards
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Scaffold` — embeds the briefing at build time and installs it beside a tree
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the skill sources under `skills/`
