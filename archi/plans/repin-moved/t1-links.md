---
node: Links
owns: [an-exact-move-repins-in-one-pass]
facts: [a-design-written-apart-from-the-code-falls-behind-it@c8d7cc, an-assistant-guesses-which-files-answer-a-written-obligation@b1667a]
---

# t1 — Links

repin --moved: the bulk consumer of exact candidates

## Spec

- `Links`
- `Service type_of Links`
- `Cli.drive consult(->Command, <-Report) Links.record`
- `Cli.drive consult(->Command, <-Report) Links.verify`
- `Links.Grader`

## Inputs

## Outputs

- crates/archi/src/links/mod.rs
- crates/archi/src/main.rs
- crates/archi/tests/link_e2e.rs
- skills/archi.md

## Stack

- the record answers where: `repin` at `crates/archi/src/links/mod.rs:1396` is the per-row
  verb the pass wraps; the grading that produces Moved-with-candidate is `check_link` plus
  `scan_for_candidate` (~line 1939), `exact: true` is the licence bit
- the pass: fold the live set, grade each link the way verify does, and where the state is
  Moved with an exact candidate, call the same journal append `repin --to` uses; print one
  row per repin in `repin`'s own render; collect inexact candidates into reported lines;
  everything else untouched; a second run finds nothing Moved and is a no-op
- the CLI arm in `crates/archi/src/main.rs` beside `repin <id>`: `--moved` and an id
  together refuse naming the two forms; usage block and module doc gain
  `link repin --moved [--json]`
- `--json`: an envelope of the repinned rows and the reported inexact ones, same data as
  the render
- one line joins the Failure modes of `skills/archi.md`: a whole file or crate renamed —
  verify grades the old anchors Moved with exact candidates, and
  `archi link repin --moved` accepts them in one pass; declare the edit to `AgentBrief`
- tests in `crates/archi/tests/link_e2e.rs` beside the repin family: a fixture with two
  linked symbols, rename the file, run the pass, assert both rows repinned and the render;
  an inexact case (edit the body while moving) reported and untouched; the no-op second
  run; the two-forms refusal; the json shape

## Verifications

### an-exact-move-repins-in-one-pass

- test — a renamed file's links repin to their exact candidates in one pass, one row each
- test — an inexact candidate is reported and not taken
- test — links grading clean, drifted or missing-without-candidate are untouched, and a
  second run is a no-op
- test — `repin <id> --moved` refuses, naming the two forms
- test — `--json` carries the same rows as the render
