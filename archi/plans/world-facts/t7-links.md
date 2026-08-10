---
node: Links
owns: [the-scenario-is-the-address-not-the-step]
---

# t7 — Links

a scenario as an addressable spec ref

## Spec

- `Links`
- `Service type_of Links`
- `Cli.drive consult(->Command, <-Report) Links.record`
- `Cli.drive consult(->Command, <-Report) Links.sweep`
- `Cli.drive consult(->Command, <-Report) Links.verify`
- `DocMint.read_links recall(<-LinkEvent) Links.serve_links`
- `Planner.coverage recall(<-LinkEvent) Links.serve_links`
- `Planner.run_capture consult(->ItemHashIndex, <-LinkEvent) Links.capture`

## Inputs

- from t1 — the scenario names inside a fact and their uniqueness rule

## Outputs

- crates/archi/src/links/mod.rs

## Stack

- `<fact-slug>#<scenario name>` joins the spec-ref parser beside element paths
- resolution reads the facts through the docs pass, not the pinned render

## Verifications

### the-scenario-is-the-address-not-the-step

- test — a link to `<fact-slug>#<scenario name>` resolves and verifies
- test — rewording a step under that scenario leaves the link untouched
- test — renaming the scenario reports the link as unresolved
- test — `link repin` moves the link onto the new scenario name
- test — two scenarios sharing a name inside one fact raise a located error
- test — `link add` naming a scenario no fact holds refuses
