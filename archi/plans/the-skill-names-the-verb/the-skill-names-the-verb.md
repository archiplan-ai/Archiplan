# the-skill-names-the-verb

The verb that lets a writer declare its own work landed, and the text that would send a
writer to it did not move. The implement skill still forbids the sub-agent every `link`
command, and still sends the orchestrator to a list of candidates that capture stopped
making three units ago. Nothing holds the text to the tool, so the drift is silent and
this is the third round of it.

## Stack

- Rust — existing codebase, `crates/archi`
- the skill sources under `skills/` — existing shape: `scaffold.rs` embeds all eight with
  `include_str!` at build time, so the text ships inside the binary
- cargo test, plain `#[test]` files — existing convention, user choice on earlier plans, no
  new dependency
- `crates/archi/tests/init_e2e.rs` — existing home: it already embeds the skill sources the
  same way for its byte-equality checks
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Scaffold` — embeds the briefing at build time and installs it beside a tree
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the skill sources under `skills/`
