---
node: Scaffold
owns: [a-skill-migrates-a-standing-project-into-the-world, the-why-reads-back-from-the-record]
facts: [what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1]
---

# t1 — Scaffold

the two pages ask only what the record does not answer

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`
- `AgentBrief`
- `Scaffold.emit persist(AgentBrief) SourceTree.store`

## Inputs

## Outputs

- skills/archi-migrate.md
- skills/archi-explain.md
- crates/archi/tests/init_e2e.rs

## Stack

- in `skills/archi-migrate.md` the world pass's interview opening ("You ask, you record,
  and you never fill an answer in yourself") becomes draft-first: the candidate is drafted
  from the prose it came from, the operator confirms or corrects, and only the gaps are
  asked — except the workaround, which is always asked and never pre-filled, because the
  gate is worth nothing answered from paper; the "Never batch them" rule and the
  second-ask-with-shapes stay
- same pass, the fact-writing step gains the anchoring move: a scenario a standing suite
  already proves is bound at the moment the fact is written —
  `archi link add "<fact>#<scenario>" <test file>#<test fn> --kind indirect` — instead of
  waiting for a plan close that may never come
- in `skills/archi-explain.md` the resolve preamble (before the chain) gains two
  sentences: read and quote the element's identity sentence — the subject before the why —
  and put a question that fits several addresses to the user as options; the chain's step
  2 names the intent-origin hop beside the stressor one (`origin: intent` answers from the
  intent folder's own problem statement); the chain's numbered order does not change
- guards in `crates/archi/tests/init_e2e.rs`: standing migrate/explain guards keep their
  assertions (`the_migration_skill_learns_to_ask` pins the second-ask; the chain-order
  guard pins six ascending commands — the resolve edits sit before the chain, so it holds);
  new tests per the four new verify bullets, two per requirement
- `scaffold.rs` is untouched: the pages are embedded by path

## Verifications

### a-skill-migrates-a-standing-project-into-the-world

- test — the world pass drafts the candidate from its prose and asks only the gaps, and
  the workaround is never answered from prose
- test — the world pass anchors a scenario a standing suite already proves, at the moment
  the fact is written

### the-why-reads-back-from-the-record

- test — the page reads the element's definition before the chain and puts an ambiguous
  question to the user as options
- test — the page names the intent-origin hop beside the stressor one
