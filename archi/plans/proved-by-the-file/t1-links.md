---
node: Links
owns: [a-declaration-names-the-test-that-proves-it]
facts: [a-design-written-apart-from-the-code-falls-behind-it@c8d7cc, an-assistant-guesses-which-files-answer-a-written-obligation@b1667a]
---

# t1 — Links

proved-by takes the file where the file is the address

## Spec

- `Links`
- `Service type_of Links`
- `Cli.drive consult(->Command, <-Report) Links.record`
- `Links.Capture`

## Inputs

## Outputs

- crates/archi/src/links/capture.rs
- crates/archi/tests/link_e2e.rs

## Stack

- two refusal sites carry the demand, same words: the verb side in `declare` — the
  `--proved-by` check near `crates/archi/src/links/capture.rs:750` ("it names a file and
  no symbol — `--proved-by` names the test itself") — and the mint-side reader near line
  569, which refuses a declaration file whose `proved_by` names a bare file
- the rule both sites learn: `code::canonicalizer_of(file)` — a `RUST_CANON` file indexes
  symbols, so a bare path keeps refusing with the standing words plus the rule ("a Rust
  test is addressable — name the test fn"); a `TEXT_CANON` file holds whole, so the bare
  path resolves the way `--symbol` already resolves one, and the minted link's `proves`
  carries the whole-file anchor
- a `file#symbol` into a TEXT_CANON file still refuses as it does today — the resolver
  cannot find what the canonicalizer does not index; the refusal may say why
- tests beside the declaration family in `crates/archi/tests/link_e2e.rs`: fixture gains a
  `.ts` source and a `.ts` test file; the three verify bullets, plus the standing Rust
  strictness re-asserted

## Verifications

### a-declaration-names-the-test-that-proves-it

- test — `--proved-by` with a bare path into a symbol-indexed file refuses, naming the rule
- test — `--proved-by` with a bare path into a whole-file-canonicalized test file is
  accepted, minted, and the reverse view shows the file as the proof
- test — the mint-side reader accepts the same bare-file proof and refuses the same
  symbol-indexed bare file
