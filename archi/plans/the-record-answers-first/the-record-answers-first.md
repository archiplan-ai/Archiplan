# the-record-answers-first

The tool answers "which files answer this part of the model" in one command, and has for
several rounds. No skill asks it. The planner guesses a task's outputs from file names,
and the sub-agent it dispatches greps the tree. Meanwhile a failure-mode line still sends
a blocked reader to review captured candidates that nothing captures.

## Stack

- Rust — existing codebase, `crates/archi`
- the skill sources under `skills/` — `scaffold.rs` embeds all nine with `include_str!` at
  build time, so the source file is the shipped text
- cargo test, plain `#[test]` files — existing convention, user choice on earlier plans
- `crates/archi/tests/init_e2e.rs` — existing home: it already embeds the skill sources the
  same way and holds the guard that reads every one of them
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Scaffold` — embeds the briefing at build time and installs it beside a tree
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the skill sources under `skills/`
