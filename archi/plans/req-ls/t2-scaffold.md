---
node: Scaffold
owns: [the-briefing-sends-the-reader-to-the-record-before-the-tree]
facts: [what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1]
---

# t2 — Scaffold

the workflow skill names req ls where it derives requirements

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`
- `AgentBrief`

## Inputs

## Outputs

- skills/archi.md
- crates/archi/tests/init_e2e.rs

## Stack

- one edit: step 4 of `skills/archi.md` ("Derive requirements") gains the read before the
  write — `archi req ls --satisfies <element>` for the elements the new claim will name,
  so the neighbouring claims are on screen before the file exists; the "Search, do not
  grep" ground rule gains `req ls` beside `world ls --covers` and `link ls --spec` as the
  structural reads that answer before search does
- the guard home is `crates/archi/tests/init_e2e.rs`; the new test follows the family that
  reads an embedded skill's passage (`passage()`, `bullet()`, `flat()` already exist)
- `scaffold.rs` is not touched: the skill sources are embedded by `include_str!`, so the
  source edit is the shipped edit

## Verifications

### the-briefing-sends-the-reader-to-the-record-before-the-tree

- test — the embedded workflow skill names `req ls --satisfies` in the passage that
  derives requirements
