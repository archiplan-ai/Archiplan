---
node: Scaffold
owns: [the-briefing-sends-the-reader-to-the-record-before-the-tree]
facts: [what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1]
---

# t1 — Scaffold

the skills ask the record which files answer a ref

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`
- `AgentBrief`
- `Data type_of AgentBrief`
- `Scaffold.emit persist(AgentBrief) SourceTree.store`

## Inputs

## Outputs

- skills/archi-plan.md
- skills/archi-implement.md
- skills/archi.md
- crates/archi/tests/init_e2e.rs

## Stack

- the read is `archi link ls --spec <ref>`, and it takes a node, a port or `req:<slug>`;
  it prints one row per recorded file, so the answer is a file list
- in `skills/archi-plan.md` the place is Step 4, the `## Outputs` bullet under
  "Tasks — one file per node", where the text today says only "the files the task will write"
- in `skills/archi-implement.md` the place is the "Sub-agents" section, the sentence listing
  what every prompt must carry
- in `skills/archi.md` the stale line is under "Failure modes": `plan next` blocked on
  coverage, telling the reader to confirm or retire candidates. The gate now names the files
  it wants and the repair is a declaration
- the guard is `no_embedded_skill_sends_the_reader_to_confirm_candidates` in
  `crates/archi/tests/init_e2e.rs`; it holds a list of forbidden phrases and reads all nine
  installed skills against their embedded copies

## Verifications

### the-briefing-sends-the-reader-to-the-record-before-the-tree

- test — the embedded plan skill names `archi link ls --spec` in the passage that authors
  `## Outputs`
- test — the embedded implement skill's sub-agent contract requires the recorded files for
  the task's refs in the prompt
- test — no embedded skill tells a reader to confirm, review or retire a candidate
