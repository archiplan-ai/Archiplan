---
node: Links
owns: [a-link-stands-asserted-or-it-does-not-stand]
facts: [a-design-written-apart-from-the-code-falls-behind-it@c8d7cc, an-assistant-guesses-which-files-answer-a-written-obligation@b1667a]
---

# t1 — Links

one standing: the guess machinery leaves the code

## Spec

- `Links`
- `Service type_of Links`
- `Cli.drive consult(->Command, <-Report) Links.record`
- `Cli.drive consult(->Command, <-Report) Links.sweep`
- `Cli.drive consult(->Command, <-Report) Links.verify`
- `DocMint.read_links recall(<-LinkEvent) Links.serve_links`
- `Planner.coverage recall(<-LinkEvent) Links.serve_links`
- `Planner.run_capture consult(->ItemHashIndex, <-LinkEvent) Links.capture`
- `Links.Grader`
- `Links.Journal`
- `Links.Capture`

## Inputs

## Outputs

- crates/archi/src/links/mod.rs
- crates/archi/src/links/capture.rs
- crates/archi/src/main.rs
- crates/archi/tests/link_e2e.rs

## Stack

- the standing is the `Standing` enum in `crates/archi/src/links/mod.rs`; `Evidence` is one
  of its variants and the whole removal hangs off it
- `confidence()` sits at `crates/archi/src/links/mod.rs:1542` and has exactly one live
  caller, the audit loop at line 2433 that walks `Standing::Evidence` rows; the `decays`
  field and the erosion constant go with it
- `Event::Decay` is read at `crates/archi/src/links/mod.rs:680` and emitted at
  `crates/archi/src/links/capture.rs:1073`; the emitter goes, and the reader stays as a
  skip so an older journal still folds
- the `--evidence` filter is the `evidence_only` parameter of `ls`, at
  `crates/archi/src/links/mod.rs:1333`; `link confirm` is documented at line 1337
- the audit's third finding and `--prune` live in the same sweep; `crates/archi/src/main.rs`
  carries the CLI surface for `confirm`, `--evidence` and `--prune`, in the arms and in the
  usage block near line 114
- an evidence row already in a journal must load as asserted, so the fold maps the standing
  on read rather than rejecting the word
- `crates/archi/tests/link_e2e.rs` drives the surface end to end; its
  `the_standing_journal_takes_its_rule_from_its_origin_with_no_migration` reads this
  project's own journal, which holds real `evidence` rows and decay events, so it is the
  natural place to prove the old journal still folds

## Verifications

### a-link-stands-asserted-or-it-does-not-stand

- test — `archi link confirm <id>` answers a usage error, and so does `archi link ls --evidence`
- test — `archi link audit --prune` answers a usage error, and the audit reports dark code and
  dark spec only
- test — a journal fixture holding an `add` with `"standing":"evidence"` and a `decay` event
  folds without error, and `link ls` prints that row as asserted
- test — `link verify` grades that row exactly as it grades an asserted one
- test — this project's own journal still loads, and no row prints the word `evidence`
