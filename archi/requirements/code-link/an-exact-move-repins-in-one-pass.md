---
kind: functional
origin: intent
satisfied-by: [Cli, Links.Grader]
deferred:
---

# An exact move repins in one pass

`archi link repin --moved [--json]` walks every live link and, where the grading finds the
anchor gone with an exact-hash candidate elsewhere — the same body at a new path — repins
to that candidate, printing one row per repin in `repin`'s own words. An inexact candidate
is reported and never taken. A link that grades anything else is untouched, and a second
run is a no-op. `--moved` takes no id: the two forms of `repin` are separate, and giving
both refuses. `--json` carries the same rows, as the house does.

## System Context

A crate rename orphans every link into it at once. The grader already proves each move —
`scan_for_candidate` matches the body hash across the tree and prints the exact new
address — but the only consumer is a person typing `link repin <id> --to` one row at a
time. On this tree that was hand-batches all week; on a whole rename it is dozens of rows,
and the session that hit it live broke instead. The knowledge is computed and then thrown
at a human: the verb is the missing consumer, nothing else is missing.

Exactness is the licence. A body-hash match is the same code at a new place, so accepting
it in bulk asserts nothing new; an inexact candidate is a judgement, and judgement stays
per-row with a person.

## Satisfy

`Cli` (the `--moved` form beside `repin <id>`, the rows, the refusal on both forms at
once). `Links.Grader` (grades the live set and serves the exact candidates the pass
accepts).

- test — a renamed file's links repin to their exact candidates in one pass, one row each
- test — an inexact candidate is reported and not taken
- test — links grading clean, drifted or missing-without-candidate are untouched, and a
  second run is a no-op
- test — `repin <id> --moved` refuses, naming the two forms
- test — `--json` carries the same rows as the render
