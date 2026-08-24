---
node: WorldDoc
owns: [a-world-fact-carries-its-scenarios, a-scenario-runs-where-its-link-points]
---

# t2 — WorldDoc

the four standing facts move to the new shape

## Spec

- `WorldDoc`
- `Data type_of WorldDoc`

## Inputs

- from t1 — the reader that decides what the four facts must look like

## Outputs

- crates/archi/src/docs/world.rs
- crates/archi/src/docs/world_check.rs
- crates/archi/src/plans/mod.rs
- archi/world/a-design-written-apart-from-the-code-falls-behind-it.md
- archi/world/an-assistant-guesses-which-files-answer-a-written-obligation.md
- archi/world/why-a-design-was-chosen-lives-in-one-person-s-memory.md
- archi/world/work-runs-in-several-directions-at-once-and-more-than-one-person-joins-it.md

## Stack

- three call sites broke when the reader's signature changed in wave 1 and must be repaired here, so the crate compiles again: `world_check.rs` drops the member argument and stops reading a feature field, `plans/mod.rs` stops reading it too
- the `Scenarios` section must reach the reader with its `###` sub-headings intact: `md::parse` lifts every `###` into its own heading, so the section arrives empty and the grammar is never called
- each `## Scenarios` block becomes `### <name>` plus its step lines
- the names stay byte-identical, because six links address them and a rename unresolves those

## Verifications

### a-world-fact-carries-its-scenarios

- test — `cargo build -p archi` succeeds: the three call sites the signature change broke are repaired
- test — unit test in `world.rs`: a fact in the new shape delivers its `Scenarios` section with every `###` heading and step line, and the grammar is called on it
- test — check_e2e: all four facts parse under the new reader with no diagnostic
- test — check_e2e: `archi check` on this tree exits 0 and reports the same four facts
- test — grep assertion: no `Feature:` and no `Scenario:` line remains under `archi/world/`

### a-scenario-runs-where-its-link-points

- test — unit test: an anchor carrying a member prefix reports that member as the scenario's runner
- test — unit test: a bare anchor reports the project's own repository
- test — grep assertion: no `@runs:` remains anywhere in the tree
