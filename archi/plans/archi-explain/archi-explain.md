# archi-explain

The tool writes the why on every step — decisions price trades, stressors sign breaks,
saves carry reasons, the world holds conditions — and no page teaches reading it back.
The old client's explain discipline is ported under the new verbs, world first, and
decisions gain the structural read the chain needs.

## Stack

- Rust — existing codebase, `crates/archi`
- the skill sources under `skills/` — `scaffold.rs` embeds them with `include_str!`
- cargo test, plain `#[test]` files — existing convention
- `crates/archi/tests/mint_e2e.rs` — existing home of the `req ls` listing tests the new
  verb mirrors; `crates/archi/tests/init_e2e.rs` — existing home of the skill guards
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Cli` — parses the verb, renders the rows and the refusal
- `DocsCompiler` — serves the standing decision set the listing reads
- `Scaffold` — embeds the briefing at build time and installs it beside a tree
- `Cli` realizes `crates/archi/src/main.rs`
- `DocsCompiler` realizes `crates/archi/src/docs/mod.rs` and `crates/archi/src/tradeoffs.rs`
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the skill sources under `skills/`
