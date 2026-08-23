---
kind: functional
origin: stressor(the-writer-names-a-port-it-did-not-answer)
satisfied-by: [Links.Capture, Links.Journal]
deferred:
---

# A declaration names the test that proves it

Every declared pair carries a third name: the test that proves the symbol answers what it
was declared to answer. The demand follows the file's canonicalizer: in a file that indexes
symbols, the test resolves to its symbol and a bare file refuses — the test itself is
addressable, so it is named; in a file the canonicalizer holds whole (every language but
Rust today), the file is the finest address that exists, and `--proved-by` takes it bare,
the same way `--symbol` already does. `plan next` refuses a test that resolves to neither. The minted link records the test beside the pair, and the reverse view
shows it, so a reader who doubts a pair has one place to go.

## System Context

The diff proves that a symbol changed and that a writer named something. It cannot prove
the naming is true, and every other check runs on the wrong side: the port resolves, the
symbol resolves, the digests match — all of it holds for a pair that answers nothing. The
writer is also the actor with an interest, since naming the nearest plausible port is the
cheapest way out of a refusal and looks exactly like care.

Naming the test does not make the claim true either. What it does is move the claim from
"I say so" to "this says so", which a reader checks in one step instead of re-deriving the
design. It also costs the writer nothing they were not already doing: the task's
verifications are the contract, and the test exists before the implementation under TDD.

What this deliberately does not do is run anything. `archi` runs no tests, here as
everywhere else. The test's existence is checked; its passing is the suite's business.

## Satisfy

`Links.Capture` (refuses a declaration whose test does not resolve).
`Links.Journal` (records the test beside the pair and serves it to the reverse view).

- test — a declaration naming a test that resolves mints a link carrying it
- test — a declaration naming a test that resolves to nothing refuses, naming the test
- test — a declaration with no test at all refuses
- test — the reverse view of a requirement names the test beside the code
- test — the test symbol may sit in any file, not only the declared output
- test — `--proved-by` with a bare path into a symbol-indexed file refuses, naming the rule
- test — `--proved-by` with a bare path into a whole-file-canonicalized test file is
  accepted, minted, and the reverse view shows the file as the proof
- test — the mint-side reader accepts the same bare-file proof and refuses the same
  symbol-indexed bare file
