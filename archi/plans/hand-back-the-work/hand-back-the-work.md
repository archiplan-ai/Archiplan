# hand-back-the-work

Answer the three breaks the first day of real use produced. A scenario link stops
witnessing more than it claims. The planning procedure stops asking for a block no verb
reads. And the step that closes a plan hands the operator the Gherkin, the state of every
link and the command that anchors it, instead of a list of identifiers.

## Stack

- Rust — existing codebase, unchanged; this unit introduces no technology
- cargo test, plain `#[test]` integration files — existing convention, no new dependency
- one scenario digest, now taking the grain as an argument — the fold from the previous unit stands, and a second digest is what the argument avoids
- no runtime infrastructure — unchanged, archi is a CLI over files and git

## Architecture

- `Links` — a scenario link witnesses the scenario it addresses and nothing else
- `Links` realizes `crates/archi/src/links/mod.rs` and the digest in `crates/archi/src/docs/world_check.rs`
- `Scaffold` — the planning procedure it installs, corrected to match what the tool does
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the embedded skill text under `skills/`
- `Planner` — the closing step renders what it already knows
- `Planner` realizes `crates/archi/src/plans/mod.rs`
