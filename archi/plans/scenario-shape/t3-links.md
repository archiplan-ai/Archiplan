---
node: Links
owns: [a-scenario-link-binds-two-hashes, the-digest-witnesses-one-scenario]
facts: [a-design-written-apart-from-the-code-falls-behind-it@4bc5db, an-assistant-guesses-which-files-answer-a-written-obligation@632182]
---

# t3 — Links

the digest reads the new parse and the six links stay clean

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

- from t1 — the parsed scenario the digest hashes
- from t2 — the four rewritten facts the six standing links address

## Outputs

- crates/archi/src/links/mod.rs
- crates/archi/src/docs/world_check.rs

## Stack

- the per-scenario digest hashes the heading name and the step lines; there is no feature line to include
- the whole-block digest changes value, because the feature line leaves the input — the plan's drift line must be re-pinned, not preserved

## Verifications

### a-scenario-link-binds-two-hashes

- test — unit test: a link over an unchanged pair verifies clean after the shape change
- test — unit test: a reworded step fails the link and names the scenario side
- test — unit test: a changed code item fails the link and names the code side
- test — e2e: `archi link verify` on this tree reports the six scenario links clean

### the-digest-witnesses-one-scenario

- test — unit test: rewording a step in a sibling scenario leaves the link clean
- test — unit test: the whole-block digest is re-pinned to its new value, and the pin test names why it moved
- test — grep assertion: exactly one Sha256 site over a parsed block in the crate
