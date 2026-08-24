# not-code-widens-the-boundary

The wave gate catches text artifacts — lockfiles, generated json — and its refusal offers
only declare-it. The second repair exists (`[audit] exclude`, the boundary capture and the
audit already share) and the message never names it, so the reader stuck on a lockfile
declares junk or stalls. The refusal learns the second sentence, and the briefing's
failure modes carry the case.

## Stack

- Rust — existing codebase, `crates/archi`
- cargo test, plain `#[test]` files — existing convention
- `crates/archi/tests/plan_e2e.rs` — home of the gate-refusal tests;
  `crates/archi/tests/init_e2e.rs` — home of the skill guards
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Planner` — the wave lifecycle and the refusals its gates print
- `Planner` realizes `crates/archi/src/plans/mod.rs`
