---
node: Planner
owns: [the-gate-refusal-names-the-repair-that-stands]
facts: [an-assistant-handed-the-whole-of-a-job-does-part-of-it@549190]
---

# t1 — Planner

the refusal names the boundary repair

## Spec

- `Planner`
- `Service type_of Planner`
- `Cli.drive consult(->Command, <-Report) Planner.advance`
- `AgentBrief`

## Inputs

## Outputs

- crates/archi/src/plans/mod.rs
- crates/archi/tests/plan_e2e.rs
- skills/archi.md
- crates/archi/tests/init_e2e.rs

## Stack

- the file gate's refusal is built in `gate_coverage` (`crates/archi/src/plans/mod.rs`),
  the message opening "this wave moved code no declaration accounts for"; it gains one
  sentence after the declare-repair block: a changed file that is not code leaves the
  scans through `[audit] exclude` in `archi.toml` — the boundary the audit and capture
  already share — and links into excluded files still verify
- existing plan_e2e tests read this refusal line by line (`the_refusal_names_the_file_and_
  no_task`, the `link add`-denial assertions) — extending is safe, replacing is not
- `skills/archi.md` Failure modes gains the wave-gate case beside the audit's prose-files
  entry: the gate names lockfiles or generated artifacts — not code motion; widen
  `[audit] exclude` once, the wave gate, capture and the audit share the boundary, and a
  link into an excluded file still verifies
- one new guard in each test home per the two verify bullets

## Verifications

### the-gate-refusal-names-the-repair-that-stands

- test — the file gate's refusal names the boundary repair: a non-code file leaves through
  `[audit] exclude`, and the sentence names the manifest key
- test — the briefing's failure modes carry the same case for the wave gate, beside the
  audit's prose-files entry
