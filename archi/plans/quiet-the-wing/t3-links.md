---
node: Links
owns: [a-scenario-link-binds-two-hashes, the-scenario-is-the-address-not-the-step]
facts: [a-design-written-apart-from-the-code-falls-behind-it@4bc5db, an-assistant-guesses-which-files-answer-a-written-obligation@632182]
---

# t3 — Links

a scenario link binds a digest per side

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

- from t1 — the scenario digest function, computed over the parsed block

## Outputs

- crates/archi/src/links/mod.rs

## Stack

- the spec-side digest rides the link record beside the code-side hashes already there
- `verify` names which side moved; `repin` binds the pair again
- the digest comes from t1's function — no second one is written here

## Verifications

### a-scenario-link-binds-two-hashes

- test — a link over an unchanged pair verifies clean
- test — a reworded step fails the link and the message names the scenario side
- test — a changed code item fails the link and the message names the code side
- test — both sides moved: the failure names both
- test — `link repin` binds the new pair and the next verify is clean
- test — a link over an element path, not a scenario, grades exactly as it does today

### the-scenario-is-the-address-not-the-step

- test — a link to `<fact-slug>#<scenario name>` resolves and verifies
- test — rewording a step leaves the address resolving, and the digest decides the link
- test — renaming the scenario reports the link as unresolved
- test — two scenarios sharing a name inside one fact raise a located error
