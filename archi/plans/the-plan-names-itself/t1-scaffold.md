---
node: Scaffold
owns: [the-plan-names-itself]
facts: [what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1]
---

# t1 — Scaffold

step 1 derives the name and asks nobody

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`
- `AgentBrief`
- `Scaffold.emit persist(AgentBrief) SourceTree.store`

## Inputs

## Outputs

- skills/archi-plan.md
- crates/archi/tests/init_e2e.rs

## Stack

- `skills/archi-plan.md` step 1 ("Name and create the plan") rewrites: derive the name
  from the problem statement — short, kebab-case, like the standing plans; a name the user
  volunteered is used as given; check `archi plan list` for a collision and derive another
  name when it hits, still without a question; no poll anywhere in the step; the rest of
  the step (`plan use`, the unsaved-model refusal, the record-folder note) stays word for
  word
- the standing guards that read the plan skill (`the_planning_skill_collects_its_scenarios…`,
  `the_planning_skill_seeds_its_outputs_from_the_record`, byte-equality, pointer,
  confirm-candidates) keep their assertions — check them before editing
- two new guards in `crates/archi/tests/init_e2e.rs` per the verify bullets, `passage()` /
  `flat()` house style

## Verifications

### the-plan-names-itself

- test — the embedded plan skill derives the name itself and its step 1 names no poll
- test — a volunteered name is used as given, and a collision derives another name
  without a question
