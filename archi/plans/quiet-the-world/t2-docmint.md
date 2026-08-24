---
node: DocMint
owns: [one-verb-mints-the-world-fact]
facts: [why-a-design-was-chosen-lives-in-one-person-s-memory@97f511]
---

# t2 — DocMint

a repeated mint converges instead of refusing

## Spec

- `DocMint`
- `Function type_of DocMint`
- `Cli.drive consult(->Command, <-Report) DocMint.mint`
- `Cli.drive consult(->Command, <-Report) DocMint.remove`

## Inputs


## Outputs

- crates/archi/src/docs/mint.rs
- crates/archi/tests/world_e2e.rs

## Stack

- the untouched-skeleton test compares against the exact bytes the mint writes
- the converge path reports `already minted` and returns success, as `req add` and `stress add` do

## Verifications

### one-verb-mints-the-world-fact

- test — a second `world add` on an untouched skeleton reports `already minted` and writes nothing
- test — a second `world add` on a file whose prose was written refuses and names it
- test — the exit code of the converge path equals the one `req add` returns for the same case
- test — world_e2e: mint, mint again, and the file on disk is byte-identical
