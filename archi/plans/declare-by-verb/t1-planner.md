---
node: Planner
owns: [the-wave-opens-a-declaration-file-for-every-task, a-verb-writes-the-declaration]
facts: [an-assistant-handed-the-whole-of-a-job-does-part-of-it@549190]
---

# t1 — Planner

the wave opens the declaration file, and a verb writes into it

## Spec

- `Planner`
- `Service type_of Planner`
- `Cli.drive consult(->Command, <-Report) Planner.advance`
- `Cli.drive consult(->Command, <-Report) Planner.author`
- `PlanFile`
- `Data type_of PlanFile`
- `Links`

## Inputs

## Outputs

- crates/archi/src/plans/mod.rs
- crates/archi/src/links/capture.rs
- crates/archi/src/main.rs
- crates/archi/tests/plan_e2e.rs

## Stack

- `plan start` writes the wave index at `crates/archi/src/plans/mod.rs:1593` through `links::capture::write_index`, and the wave close writes the next one at line 1740; the declaration files are written in the same two places, one per task the wave puts in flight
- the reader is `read_declarations` in `crates/archi/src/links/capture.rs`, and it already distinguishes an absent file from one that parses and names nothing — the absent arm becomes unreachable for a wave opened after this change and stays for one opened before it
- the verb parses in `crates/archi/src/main.rs` beside the other `plan task` subcommands, and it resolves through the paths `link add` already uses: `Anchor::parse` plus `resolve_anchor` for the two anchors, `SpecRef::parse` plus the model and requirement sets for the ref
- the entry is appended to the task's file as TOML written by the tool, so the shape cannot drift from what the reader accepts
- `archi batch -` re-invokes the binary per line and stops at the first refusal, so a batch of these needs nothing new: `crates/archi/src/batch.rs:75` already runs any verb but `batch` itself
- `plan reset` clears the wave's state and takes the declaration files with it

## Verifications

### the-wave-opens-a-declaration-file-for-every-task

- test — opening a wave writes one file per task in flight, beside the wave index
- test — the file carries the shape as comments and declares nothing
- test — the close refuses it as a file that declares nothing, not as an absent one
- test — `plan reset` removes the files with the rest of the wave's state
- test — a wave opened before this change, whose files are absent, still refuses and says so

### a-verb-writes-the-declaration

- test — the verb appends one entry and the wave then closes on it
- test — a symbol that resolves to nothing refuses, naming the symbol, and writes nothing
- test — the same for a test that resolves to nothing, and for a ref that names neither an element nor a requirement
- test — the refusal says which of the three failed
- test — a repeated identical entry appends no second copy and says the entry stands
- test — several entries land through `archi batch -`, and a refusal mid-batch leaves the earlier entries written
- test — the verb refuses outside a started wave, naming the lifecycle step that opens one
