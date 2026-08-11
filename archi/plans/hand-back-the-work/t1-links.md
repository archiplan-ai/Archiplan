---
node: Links
owns: [the-digest-witnesses-one-scenario]
facts: [a-design-written-apart-from-the-code-falls-behind-it@4bc5db, an-assistant-guesses-which-files-answer-a-written-obligation@632182]
---

# t1 — Links

the digest narrows to the scenario the link addresses

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


## Outputs

- crates/archi/src/links/mod.rs
- crates/archi/src/docs/world_check.rs

## Stack

- `scenario_digest` takes the scenario to fingerprint; naming none keeps the whole-block grain the plan's drift line wants
- the link passes the scenario its ref addresses, so a sibling moving changes nothing
- one function, one contract — a second digest is what the argument exists to avoid

## Verifications

### the-digest-witnesses-one-scenario

- test — unit test in `links/mod.rs`: a fact of two scenarios, a link on the first, a step reworded in the second — the grade stays `clean`
- test — unit test in `links/mod.rs`: the same fact, a step reworded in the addressed scenario — the grade is `scenario-drifted` and the message names the scenario side
- test — unit test in `links/mod.rs`: renaming the addressed scenario yields `spec-drifted`, not a digest failure
- test — unit test in `world_check.rs`: `scenario_digest` with no scenario named returns the value the plan's drift line pinned before this change
- test — grep assertion in the test: exactly one Sha256 site over a parsed block in the crate
