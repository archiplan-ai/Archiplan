# bullets-may-wrap

Two rules nobody chose. A bullet in a plan record is refused the moment its text runs onto a
second line, although that is valid markdown and every other reader accepts it — the
workaround was a sentence in the planning skill telling the author not to wrap, the tool
asking the writer to work around it. And the close refuses an empty scenario block on
emptiness alone, so a plan that owed no condition was sent to record one under a payload,
which is the opposite of the design. A bullet section is read whole instead, split at its
`- ` lines; and the empty block asks only where a condition is owed.

## Stack

- Rust — existing codebase, `crates/archi`
- cargo test, plain `#[test]` files — existing convention, no new dependency
- `crates/archi/src/plans/records.rs` unit tests and `crates/archi/tests/plan_e2e.rs` — existing homes, user choice: no new suite
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Planner` — reads the plan's records, cuts the waves and refuses what it cannot hold
- `Planner` realizes `crates/archi/src/plans/mod.rs` and `crates/archi/src/plans/records.rs`
