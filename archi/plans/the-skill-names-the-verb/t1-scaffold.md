---
node: Scaffold
owns: [the-sub-agent-posts-its-declarations-before-it-returns]
facts: [what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1]
---

# t1 — Scaffold

the implement skill sends the writer to the verb, and tests hold it there

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`
- `AgentBrief`
- `Data type_of AgentBrief`
- `Scaffold.emit persist(AgentBrief) SourceTree.store`

## Inputs

## Outputs

- skills/archi-implement.md
- crates/archi/tests/init_e2e.rs

## Stack

- the text to change is `skills/archi-implement.md`; `crates/archi/src/scaffold.rs:19`
  embeds all eight skills with `include_str!`, so the source file is the shipped text and
  no second copy exists
- the sub-agent contract is the lettered list under "The per-task contract" (~line 134); a
  new item sends the writer to `archi plan task <id> link add --symbol --answers
  --proved-by`, several through `archi batch -`, as the last act of the task
- the orchestrator-only rule sits under "Sub-agents" (~line 240, "Every `plan` and `link`
  command stays with you"); it keeps its force and names this one verb as the exception
- the wave-loop bullet at ~line 167 still describes the shared-term capture: it sends the
  reader to `archi link ls --evidence`, `link confirm` and `link rm` on candidates that
  `plan next` no longer proposes. It is replaced by what the gate does now — it reads the
  declaration file and refuses the wave that has none
- the tests go in `crates/archi/tests/init_e2e.rs`, which already reads the skill sources
  the same way: `const SKILL_ARCHI: &str = include_str!("../../../skills/archi.md")` at
  line 35. A new const for `archi-implement.md` follows that shape
- the last test reads every one of the eight embedded skills, so a stale phrase in any of
  them fails, not only in this one

## Verifications

### the-sub-agent-posts-its-declarations-before-it-returns

- test — the embedded implement skill names the verb with all three flags: `--symbol`,
  `--answers` and `--proved-by`
- test — the embedded implement skill names `archi batch -` in the sub-agent contract
- test — the embedded implement skill still keeps `plan` and `link` with the orchestrator,
  and names this verb as the one exception
- test — no embedded skill names `link ls --evidence` or `link confirm`
