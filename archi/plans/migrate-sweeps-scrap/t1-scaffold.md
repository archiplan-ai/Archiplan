---
node: Scaffold
owns: [one-door-migrates-the-standing-project]
facts: [what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1]
---

# t1 — Scaffold

the scrap pass joins the migrate page

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`
- `AgentBrief`
- `Scaffold.emit persist(AgentBrief) SourceTree.store`

## Inputs

## Outputs

- skills/archi-migrate.md
- crates/archi/tests/init_e2e.rs

## Stack

- the triage head of `skills/archi-migrate.md` gains the third measurement:
  `ls -d archi/plans/*/waves` — a hit under a plan that `archi plan list` shows completed
  is scrap from a binary older than `the-plan-cleans-up-after-itself`; run the scrap pass
- the pass is a short third `##` section after the journal pass: confirm the plan's state
  in `archi plan list` (completed — including old `plan.json` plans), `git rm -r` the
  folder, leave any draft or started plan's folder standing, `archi check`, one commit
  naming the count; the record of what those waves accounted for is the journal, and the
  page says so
- the guard `the_migrate_page_opens_with_the_measurements_and_carries_both_passes` in
  `crates/archi/tests/init_e2e.rs` widens: third measurement present, the scrap section
  ordered after the journal pass, and the live-plan exemption stated; rename the test if
  its name now under-claims

## Verifications

### one-door-migrates-the-standing-project

- test — the embedded `archi-migrate` opens with the measurements, carries the world,
  journal and scrap passes, and names `archi-migrate-fractal` as the old client's own page
- test — the scrap pass says a completed plan's `waves/` goes and a live plan's stays
