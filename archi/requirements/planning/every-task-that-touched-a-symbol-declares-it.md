---
kind: functional
origin: stressor(two-tasks-change-one-symbol-and-each-expects-the-other-to-declare-it)
satisfied-by: [Links.Capture, Planner]
deferred:
---

# Every task that touched a symbol declares it

When more than one in-flight task claims the file a changed symbol sits in, every one of
those tasks declares that symbol, and the wave refuses while any of them has not. The
refusal names the symbol once and every task that owes it. Each declaration mints its own
link, so a symbol two tasks answered carries two pairs and says so.

## System Context

The delta is one set for the whole wave; the declarations are one file per task. Left
unstated, a symbol two tasks touched is declared by either, by both, or by neither, and each
reading fails differently. Neither: both sub-agents assumed the other owned it. Either: the
first file to name it satisfies the gate and the second task's contribution leaves no
record, attributing shared work to whoever finished first. Both is the only reading that
records what happened.

The cost is real and is the point. Two pairs on one symbol read as two claims, not as one
duplicated, because the reverse view names the task behind each. Capture already knows which
files more than one task claims — it marks them `shared` — so the set is not new work, only
a decision that now has consequences.

## Satisfy

`Links.Capture` (derives who owes a declaration from the claim map it already builds).
`Planner` (refuses while any claimant has not declared, naming the symbol and every task).

- test — two tasks claiming one file and both changing one symbol must both declare it
- test — one of the two declaring is not enough, and the refusal names the other task
- test — both declaring mints two links on the one symbol
- test — the two links each name their own task in the reverse view
- test — a symbol only one in-flight task claims is owed by that task alone
