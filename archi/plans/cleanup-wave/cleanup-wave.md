# cleanup-wave

Parallel waves can give birth to one mechanism twice. The plan
lifecycle gains a cleanup stage between the last wave and the
scenarios: one sub-agent sweeps the unit's delta and folds the twins,
so the scenarios always bless folded code.

## Stack

- Rust — the repository's standing stack
- cargo test — the repository's standing test harness

## Architecture

- `Planner` — the cleanup stage and its latch
- `Planner` realizes crates/archi/src/plans/mod.rs
- `Scaffold` — the sweep contract in the briefing
- `Scaffold` realizes skills via include_str
