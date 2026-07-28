# plan-reads

`plan show <name>` renders any plan — record or legacy — without activation:
no `.current` write, answers on an unbound checkout. Nameless keeps the active
plan.

## Stack

- Rust — the repository's standing stack
- cargo test — the repository's standing test harness

## Architecture

- `Cli` — the optional name on plan show, a free read at the router
- `Cli` realizes crates/archi/src/main.rs
