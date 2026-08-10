---
affects: [Planner, WorldDoc, PlanState]
outcome: breaking
---

# The block moves under the plan

A plan pins a version and projects it for weeks. The facts it collects live in
`archi/world/`, which the archive does not seal — the render carries the model alone.
So a fact can be written, edited or retired at any point while the plan runs, and the
block the close prints is whatever the tree holds at the moment `plan next` reaches
the last step. The plan's one guarantee — that it projects a fixed spec — does not
reach the half of the spec it now closes on.

## Attractor

Two operators run `plan next` on the same completed plan a day apart and see different
exit ceremonies. The latch says the block was displayed, and nobody can say which
block. `plan repin` exists to move a plan onto a new version deliberately, and this
moves it without the verb, without a record and without a diff to read.

## Resolution

The block is collected at the close and the close names its drift against what the tasks
carried: derived `the-close-re-reads-the-wing-and-says-what-moved`. A copy into the plan
record was refused — it would put one scenario in two files and make the plan the second
place a person edits when a fact moves, which is the drift the wing exists to remove. A
report costs nothing to keep true, and `plan verify` already flags every task whose
obligations no longer hold; a covering fact that moved is one of those.
