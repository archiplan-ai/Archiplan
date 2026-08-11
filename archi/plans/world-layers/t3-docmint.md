---
node: DocMint
owns: [one-verb-mints-the-world-fact, the-wing-arrives-without-noise]
facts: [why-a-design-was-chosen-lives-in-one-person-s-memory@e3b0c4]
---

# t3 — DocMint

world add mints into facts

## Spec

- `DocMint`
- `Function type_of DocMint`
- `Cli.drive consult(->Command, <-Report) DocMint.mint`
- `Cli.drive consult(->Command, <-Report) DocMint.remove`

## Inputs

- from t1 — where a fact lives, so the mint writes where the walk looks

## Outputs

- crates/archi/src/docs/mint.rs
- crates/archi/tests/world_e2e.rs

## Stack

- the skeleton path becomes `archi/world/facts/<slug>.md`, created with its parents
- the removal's three pre-flights are unchanged; only the path moves

## Verifications

### one-verb-mints-the-world-fact

- test — world_e2e: `world add` writes under `archi/world/facts/` and `world ls` lists it
- test — world_e2e: a repeated `world add` on the untouched skeleton still converges
- test — world_e2e: `world rm` retires a fact from its new home

### the-wing-arrives-without-noise

- test — world_e2e: `world add` on a tree with no `archi/world/` creates `facts/` and its parent
- test — check_e2e: a tree with no `archi/world/` passes check with no world finding
