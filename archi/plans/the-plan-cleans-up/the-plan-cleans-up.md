# the-plan-cleans-up

A closed wave's working files are dead — capture reads only the current wave's index, and
a declaration is consumed once. They stayed on disk anyway: 13M across 69 dead indexes
until a hand sweep took them. The close learns to delete what it consumed, so the sweep
never needs to happen again.

## Stack

- Rust — existing codebase, `crates/archi`
- cargo test, plain `#[test]` files — existing convention, user choice on earlier plans
- `crates/archi/tests/plan_e2e.rs` — existing home: it drives the wave loop end to end
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Planner` — the wave lifecycle: open, close, complete, reset
- `Planner` realizes `crates/archi/src/plans/mod.rs`
