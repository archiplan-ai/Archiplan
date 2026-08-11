---
kind: functional
origin: stressor(two-thousand-candidates-outlive-the-matcher-that-made-them)
satisfied-by: [Links.Journal, Links.Grader]
deferred:
---

# The journal says which rule made a row

Every row records which rule produced it: inferred from shared terms, declared by a writer,
or authored by hand. `link ls`, the audit and the reverse view carry that word, and the
reverse view answers from declared and authored rows alone. Rows inferred before the rule
changed keep standing and keep grading; nothing retires them in bulk.

## System Context

The journal is append-only truth, so the 2036 inferred candidates do not go away when the
rule that made them stops governing. They grade on every verify, they decay, and 229 of them
already sit below the confidence floor with the audit asking about each one. Left
unmarked they are indistinguishable from rows a writer stood behind, and the reverse view —
the thing this round adds and a reader will trust — would compute over a set that is mostly
guesses.

Marking rather than retiring is the whole of the answer. A mass edit of append-only truth is
the one operation this design has always refused, and it would be refused here for the same
reason: the rows are a record of what the tool believed, and a record that can be rewritten
when it becomes inconvenient is not a record. Marked, they stay readable as history and stop
being read as evidence.

## Satisfy

`Links.Journal` (records the producing rule on every row and serves it).
`Links.Grader` (carries the word into `ls` and the audit, and computes the reverse view from
declared and authored rows alone).

- test — a row minted from a declaration reads as declared
- test — a row minted by inference reads as inferred, including rows written before the rule
  changed
- test — a row from `link add` reads as authored
- test — the reverse view of an element omits inferred rows and names the rest
- test — no verb retires rows in bulk by their producing rule
