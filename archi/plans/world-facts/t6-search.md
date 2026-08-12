---
node: Search
owns: [search-reaches-the-new-world]
---

# t6 — Search

world facts in the ranked corpus

## Spec

- `Search`
- `Function type_of Search`
- `Cli.drive consult(->ModelGraph, <-SearchReport) Search.find`

## Inputs

- from t1 — the parsed record, for the card's name, paragraph and killer

## Outputs

- crates/archi/src/search.rs

## Stack

- one card per fact from the same scan that reads requirements and stressors
- the scenario blocks stay out of the card body
- `--kind world` joins the existing narrowing flag and the JSON hit kinds

## Verifications

### search-reaches-the-new-world

- test — search_e2e: a phrase from a fact's name returns that fact with its slug and path
- test — search_e2e: `--kind world` returns world facts only
- test — search_e2e: the JSON envelope carries `world` as a hit kind
- test — search_e2e: a phrase appearing only inside a scenario step returns no world hit
