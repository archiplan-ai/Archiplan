---
node: WorldDoc
owns: [a-world-fact-carries-its-scenarios]
---

# t1 — WorldDoc

the workaround takes the killer's slot, and the four facts move with the shape

## Spec

- `WorldDoc`
- `Data type_of WorldDoc`

## Inputs


## Outputs

- crates/archi/src/docs/world.rs
- crates/archi/src/docs/mint.rs
- crates/archi/tests/util/mod.rs
- archi/world/facts/a-design-written-apart-from-the-code-falls-behind-it.md
- archi/world/facts/an-assistant-guesses-which-files-answer-a-written-obligation.md
- archi/world/facts/why-a-design-was-chosen-lives-in-one-person-s-memory.md
- archi/world/facts/work-runs-in-several-directions-at-once-and-more-than-one-person-joins-it.md
- archi/world/facts/what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it.md

## Stack

- `## What people do instead` joins the required sections beside the conditioning paragraph and `## Scenarios`
- `## What kills this` earns a named refusal rather than being read as an unknown heading: it is what this replaced
- the shape and the four files land in one task, because a check that outran its records would empty the wing between commits
- each fact keeps its slug and its scenario names byte-identical — six links address them
- two more places write the shape and must flip with the reader: the mint's skeleton, or `world add` creates a file the checker refuses, and the shared test fixture, or fifteen suites go red

## Verifications

### a-world-fact-carries-its-scenarios

- test — unit test in `world.rs`: a file with name, paragraph, workaround and one scenario parses clean
- test — unit test: a missing `What people do instead` raises a located `E_DOC` at the line it should open
- test — unit test: a `What kills this` heading raises a located error naming the section that replaced it
- test — unit test: a missing conditioning paragraph and an empty `Scenarios` still raise their located errors
- test — unit test: `Open questions` stays optional, present-and-empty or absent
- test — check_e2e: the four rewritten facts parse with no diagnostic and `archi check` exits 0
- test — e2e: `archi link verify` reports the six scenario links clean after the rewrite
- test — grep assertion: no file under `archi/world/facts/` holds a `What kills this` heading
- test — grep assertion: no fact names a person or carries a quoted phrase from one
- test — world_e2e: `world add` mints a skeleton the checker accepts, with the workaround heading and no killer
