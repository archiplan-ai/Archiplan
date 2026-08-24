---
node: WorldDoc
owns: [a-world-fact-carries-its-scenarios]
---

# t2 — WorldDoc

the four standing facts move into facts and lose their borrowed sources

## Spec

- `WorldDoc`
- `Data type_of WorldDoc`

## Inputs

## Outputs

- archi/world/facts/a-design-written-apart-from-the-code-falls-behind-it.md
- archi/world/facts/an-assistant-guesses-which-files-answer-a-written-obligation.md
- archi/world/facts/why-a-design-was-chosen-lives-in-one-person-s-memory.md
- archi/world/facts/work-runs-in-several-directions-at-once-and-more-than-one-person-joins-it.md

## Stack

- this task and t1 land in one wave on purpose: the reader learns the new home and the files arrive there in the same commit, or the world is empty in between
- the four files move from `archi/world/` into `archi/world/facts/`, contents otherwise untouched
- each `sources` list empties: every entry named an intent, which the world may no longer reach
- slugs and scenario names stay byte-identical, because six links address them

## Verifications

### a-world-fact-carries-its-scenarios

- test — check_e2e: all four facts parse from `facts/` with no diagnostic
- test — e2e: `archi link verify` reports the six scenario links clean after the move
- test — grep assertion: no file remains directly under `archi/world/`
- test — grep assertion: no `sources` entry in the tree names a path outside `archi/world/`
