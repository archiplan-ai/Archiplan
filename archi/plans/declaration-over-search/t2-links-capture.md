---
node: Links.Capture
owns: [the-writer-declares-what-the-code-answers, a-declaration-names-the-test-that-proves-it, the-declaration-refusal-repairs-without-guessing]
facts: [an-assistant-guesses-which-files-answer-a-written-obligation@b1667a]
---

# t2 — Links.Capture

read the declarations instead of inferring, and refuse what does not repair in one read

## Spec

- `Links.Capture`
- `Links`
- `Links.Journal`
- `Planner.run_capture consult(->ItemHashIndex, <-LinkEvent) Links.capture`

## Inputs

- from t1 — the `req:` spec-ref shape, so a declaration naming a requirement resolves, and the producing-rule field on the journal row, so a minted pair can be stamped declared

## Outputs

- crates/archi/src/links/capture.rs
- crates/archi/tests/link_e2e.rs

## Stack

- the declaration file is `archi/plans/<plan>/waves/w<NN>.<task>.declares.toml`, beside the index the wave already writes
- one array of tables, `[[declares]]`, each with `symbol`, `answers` and `proved_by`, all three required and no unknown key tolerated
- `symbol` and `proved_by` parse through the existing `Anchor::parse`, so a member qualifier and a `#symbol` suffix behave exactly as they do in `link add`
- `answers` parses through `SpecRef::parse`, and an edge is refused by matching the canonical edge form before resolution, so the refusal can name the port behind it
- the toml crate reports a span for every error; the refusal quotes the line, what was expected and what stood there
- capture keeps its diff and its claim map untouched; what changes is that the mint comes from the file and the term matching no longer mints anything

## Verifications

### the-writer-declares-what-the-code-answers

- test — a declared port becomes an asserted link on the symbol that named it, and reads as declared
- test — a declared name that resolves against neither the model nor the requirement set refuses, naming the name and the file line
- test — an edge in `answers` is refused and the refusal names the port behind it
- test — no link is minted for a symbol no declaration names, and the shared-term rule mints nothing at all

### a-declaration-names-the-test-that-proves-it

- test — a declaration naming a test that resolves mints a link carrying the test
- test — a declaration naming a test that resolves to nothing refuses, naming the test
- test — the test symbol may sit in a file outside the task's declared outputs and still resolves
- test — the minted row carries the test where `link ls --spec req:<slug>` prints it

### the-declaration-refusal-repairs-without-guessing

- test — a missing required field is refused with its line, what was expected and what stood there
- test — an unknown key is refused rather than ignored, and the refusal names the key
- test — the refusal names the task and the path of the file
- test — no refusal this task raises consists only of a restatement of the grammar
- test — a file with every field present and no unknown key is accepted with no warning
