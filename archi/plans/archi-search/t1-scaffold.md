---
node: Scaffold
owns: [the-search-doctrine-lives-in-one-skill]
facts: [what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1]
---

# t1 — Scaffold

the archi-search page, the pointers, the guards

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`
- `AgentBrief`
- `Scaffold.emit persist(AgentBrief) SourceTree.store`

## Inputs

## Outputs

- skills/archi-search.md
- skills/archi.md
- skills/archi-plan.md
- skills/archi-implement.md
- skills/archi-merge.md
- skills/archi-migrate-fractal.md
- skills/archi-migrate-world.md
- skills/archi-migrate-links.md
- crates/archi/src/scaffold.rs
- crates/archi/tests/init_e2e.rs

## Stack

- the record answers where (`link ls --spec`): `Scaffold` ← `crates/archi/src/scaffold.rs#init`,
  `AgentBrief` ← `skills/archi-implement.md`; the embedded array is `SKILLS` at
  `scaffold.rs:19` — nine entries today, `archi-search` makes ten
- the freshness header every skill opens with is the shape to copy for the new page; the
  new skill is doctrine, not workflow — short, one screen
- the pointer line carries the bare name `archi-search` and no retrieval vocabulary; the
  workflow steps keep their command mentions where a step uses one (`req ls --satisfies`
  in step 4 of `archi.md`, `link ls --spec` in the plan skill's Outputs bullet and the
  implement skill's prompt rule — those stay, the guards from earlier units read them)
- `archi.md`'s "Search, do not grep" ground-rule paragraph shrinks to the pointer; the
  phrase and the ordered list move into `archi-search.md`
- the install count in `init_e2e.rs` (`created.len()`) grows by one; the byte-equality
  loop gains the new const; `ste-writing` is exempt from the pointer rule and the test
  says so

## Verifications

### the-search-doctrine-lives-in-one-skill

- test — a fresh init installs `archi-search` byte-equal to the embedded copy
- test — the embedded `archi-search` names the order: `query --top`, then
  `req ls --satisfies`, `world ls --covers`, `link ls --spec`, then search, and says
  grep misses the model
- test — every working skill names `archi-search`, and `ste-writing` is exempt
- test — the phrase `Search, do not grep` and the doctrine's order live only in
  `archi-search`
