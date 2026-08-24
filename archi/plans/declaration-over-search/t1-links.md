---
node: Links
owns: [a-requirement-is-addressable-in-the-journal, the-journal-says-which-rule-made-a-row]
facts: [a-design-written-apart-from-the-code-falls-behind-it@c8d7cc, an-assistant-guesses-which-files-answer-a-written-obligation@b1667a]
---

# t1 — Links

a requirement gets an address and every row says which rule made it

## Spec

- `Links`
- `Service type_of Links`
- `Cli.drive consult(->Command, <-Report) Links.record`
- `Cli.drive consult(->Command, <-Report) Links.sweep`
- `Cli.drive consult(->Command, <-Report) Links.verify`
- `DocMint.read_links recall(<-LinkEvent) Links.serve_links`
- `Planner.coverage recall(<-LinkEvent) Links.serve_links`
- `Planner.run_capture consult(->ItemHashIndex, <-LinkEvent) Links.capture`
- `Links.Journal`
- `Links.Grader`

## Inputs

## Outputs

- crates/archi/src/links/mod.rs
- crates/archi/tests/link_e2e.rs

## Stack

- `SpecRef::parse` at `crates/archi/src/links/mod.rs:133` takes two shapes today; the `req:` prefix is tried before the `#` split so a requirement slug never falls to the element branch
- the requirement set of the pinned version is what `req:` resolves against — the same set `plan verify` already reads for `owns:`
- a `req:` ref carries no version pin: `<element>[@version]` keeps its `@` handling and `req:` refuses one, because a requirement slug is stable across versions by the rule that governs plan ownership
- the producing rule is a new field on the journal row, defaulting to inferred when absent, so the rows written before this change read correctly with no migration
- `link ls`, `link audit` and `link ls --spec` carry the word; `--spec req:<slug>` is the reverse view and needs no new verb

## Verifications

### a-requirement-is-addressable-in-the-journal

- test — `link add 'req:<slug>' <file#symbol>` resolves against the pinned version's requirement set and mints a row
- test — `req:` naming no requirement refuses with a message that names the slug
- test — a bare slug with no prefix still falls to the element branch and its message is byte-identical to today's
- test — `req:<slug>@v0025` refuses, and the refusal says a requirement carries no version pin
- test — the code side moving fails a `req:` link exactly as it fails an element link, and `link repin` binds it again
- test — retiring the requirement makes the link unresolvable, a state distinct from missing

### the-journal-says-which-rule-made-a-row

- test — a row minted from a declaration reads as declared in `link ls`
- test — a row carrying no rule takes it from its origin: captured reads inferred, authored reads authored, and no migration runs
- test — the standing journal's captured rows read inferred and its `link add` rows read authored, proven against the real journal
- test — a row from `link add` reads as authored
- test — `link audit` carries the word on every line it prints
- test — `link ls --spec req:<slug>` lists declared and authored rows and omits inferred ones
- test — no verb retires rows in bulk by their producing rule
