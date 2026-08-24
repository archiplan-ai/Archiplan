---
node: Planner
owns: [an-undeclared-change-refuses-the-wave, a-drifted-declaration-refuses-the-wave-that-moved-it]
facts: [an-assistant-handed-the-whole-of-a-job-does-part-of-it@549190]
---

# t3 — Planner

the wave refuses on an undeclared change, an unowed symbol and a drifted declaration

## Spec

- `Planner`
- `Service type_of Planner`
- `Cli.drive consult(->Command, <-Report) Planner.advance`
- `Cli.drive consult(->Command, <-Report) Planner.author`
- `Links.Capture`
- `Links.Grader`
- `Planner.run_capture consult(->ItemHashIndex, <-LinkEvent) Links.capture`
- `Planner.coverage recall(<-LinkEvent) Links.serve_links`

## Inputs

- from t1 — the producing-rule field, so the drift refusal can separate declared rows from the inferred and authored ones it must leave alone
- from t2 — the parsed declaration set per task and the undeclared-symbol set, which the gate turns into refusals

## Outputs

- crates/archi/src/plans/mod.rs
- crates/archi/tests/plan_e2e.rs

## Stack

- the gate sits where `gate_coverage` is called at `crates/archi/src/plans/mod.rs:1727`, before `plan.closed_waves = wave`, so a refusal leaves the wave open exactly as the coverage gate does today
- the owing set comes from the claim map capture already builds: the tasks whose `## Outputs` claim the changed symbol's file, which is the same map that marks a file `shared`
- a changed file no in-flight task claims stays a leftover note and owes no declaration
- the drift refusal reads only rows whose producing rule is declared; the advisory rows standing in this tree are inferred and must not start refusing
- `plan reset` clears the new latches as it clears the existing ones

## Verifications

### an-undeclared-change-refuses-the-wave

- test — a wave with a task whose declaration file is absent refuses, naming the task and the path it owes
- test — a file that parses and declares nothing refuses the same way
- test — a file with one entry closes the wave, whatever else the delta moved
- test — the refusal names the next command
- test — a wave with no task in flight owes no file

### a-drifted-declaration-refuses-the-wave-that-moved-it

- test — a wave that moves a declared symbol refuses, naming the link and the symbol
- test — the refusal names both exits: repin, or declare again
- test — repinning clears the refusal and the wave closes
- test — an inferred or authored row that drifts does not refuse
- test — a declared row whose code did not move does not refuse
