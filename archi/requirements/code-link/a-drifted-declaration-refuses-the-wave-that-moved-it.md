---
kind: functional
origin: stressor(the-declaration-is-true-when-written-and-false-by-the-next-wave)
satisfied-by: [Links.Grader, Planner]
deferred:
---

# A drifted declaration refuses the wave that moved it

A declared link whose code side moved refuses the wave that moved it. The refusal names the
link, the symbol and the two ways out: repin the pair, or declare the symbol again. Links
that were never declared — the inferred rows the journal carries as history, and hand-authored
rows — keep grading exactly as they do today, so drift stays advisory everywhere it already was.

## System Context

The declaration file is consumed once and never read again; the link is what survives. So a
later wave that rewrites the symbol for an unrelated reason leaves a link standing on a pair
that no longer holds, and the only signal is drift — advisory, 184 rows of it in this tree,
none failing.

Under inference nobody leaned on those rows. Under declaration the pair is the record of
what the code is for, and the reverse view is rendered from it, so a stale declared row is
read and believed. That is the failure this tool exists to prevent — a written claim the
code has quietly left behind — arriving with more authority than before.

Refusing at the wave that moved it puts the repair where the knowledge is. The person who
just rewrote the symbol knows whether it still answers what it answered; nobody reading the
audit three months later does. Confining the refusal to declared links is what keeps the
change from turning 184 standing advisory rows into a wall on the next wave anybody runs.

## Satisfy

`Links.Grader` (separates declared rows from the rest and fails a declared row whose code
side moved). `Planner` (refuses the wave, names the link, the symbol and both exits).

- test — a wave that moves a declared symbol refuses, naming the link and the symbol
- test — the refusal names both exits: repin, or declare again
- test — repinning clears the refusal and the wave closes
- test — a captured or hand-authored row that drifts does not refuse
- test — a declared row whose code did not move does not refuse
