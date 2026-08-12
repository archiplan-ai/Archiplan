# declare-by-verb

The declaration is a file somebody types, so a parser stands between the writer and the
record and speaks after the writer has gone: a task agent that misplaces a quote learns
nothing, because the refusal reaches the orchestrator hours later. And the absent file and
the empty file are two refusals for one situation, a task that accounted for nothing. The
wave now opens a file for every task it starts, and a verb appends entries into it,
resolving all three addresses before it writes.

## Stack

- Rust — existing codebase, `crates/archi`
- toml — already a dependency; the tool writes the entries, nobody types them
- cargo test, plain `#[test]` files — existing convention, no new dependency
- `crates/archi/tests/plan_e2e.rs` and the unit tests beside the code — existing homes, user choice: no new suite
- `archi batch -` — existing verb, needs nothing new: it re-invokes the binary per line and stops at the first refusal
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Planner` — the wave lifecycle and the verbs that author into it
- `Planner` realizes `crates/archi/src/plans/mod.rs` and its arm of `crates/archi/src/main.rs`
