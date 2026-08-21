# the-node-question

Facts grow from what the operator said, and the save gate goes quiet at the first cover —
so the condition a layer exists for, too obvious to say aloud, is never minted until the
operator prods by hand. The prod becomes one question the briefing asks of every node,
drafted by the agent and judged by the operator through options.

## Stack

- Rust — existing codebase, `crates/archi`
- the skill sources under `skills/` — `scaffold.rs` embeds them with `include_str!`
- cargo test, plain `#[test]` files — existing convention
- `crates/archi/tests/init_e2e.rs` — existing home of the skill guards
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Scaffold` — embeds the briefing at build time and installs it beside a tree
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the skill sources under `skills/`
