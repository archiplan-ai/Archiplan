---
node: Search
owns: [a-node-with-no-prose-speaks-through-its-ports]
---

# t1 — Search

a definitionless node is indexed by its ports' prose at summary weight

## Spec

- `Search`
- `Function type_of Search`
- `Cli.drive consult(->ModelGraph, <-SearchReport) Search.find`

## Inputs

## Outputs

- crates/archi/src/search.rs

## Stack

- the card is built at `crates/archi/src/search.rs:556-575`: the node's own definition goes to field 1 with `c.push(1, 0, doc)`, port names to field 1, and port definitions to field 2 with `c.push(2, 0, format!("{p} {doc}"))`
- `const WEIGHTS: [f64; 3] = [3.0, 2.0, 1.0]` at line 142 — name, summary, body
- the change is one branch: when the node carries no definition, its port definitions push to field 1 instead of field 2; when it carries one, nothing moves
- the display already falls back to the matching field, so a definitionless node's card prints its port prose today and keeps printing it

## Verifications

### a-node-with-no-prose-speaks-through-its-ports

- test — a node with ports and no definition ranks on its port prose at summary weight
- test — the same node loses to a name match, which still outweighs a summary
- test — a node that carries its own definition keeps its ports at body weight
- test — a node with no ports and no definition scores on its name alone, as it does today
- test — the card of a definitionless node shows its port prose, as it does today
