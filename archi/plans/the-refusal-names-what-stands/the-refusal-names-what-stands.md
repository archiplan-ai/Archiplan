# the-refusal-names-what-stands

The wave gate refuses on coverage and tells the reader to review a list of candidates and
confirm the load-bearing ones. Capture stopped proposing candidates three units ago, so the
list is always empty and the confirm has nothing to raise. The refusal is read at the one
moment somebody is stuck, and it sends them at a mechanism that is gone.

## Stack

- Rust — existing codebase, `crates/archi`
- cargo test, plain `#[test]` files — existing convention, user choice on earlier plans, no
  new dependency
- `crates/archi/tests/plan_e2e.rs` — existing home: it already drives `plan next` to the
  coverage refusal and reads its text
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Planner` — the wave lifecycle, its gates and the refusals they print
- `Planner` realizes `crates/archi/src/plans/mod.rs`
