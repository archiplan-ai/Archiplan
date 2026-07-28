---
affects: [Planner, PlanFile]
outcome: surviving
---

# A renamed task file breaks the dag

The task id lives in the filename; a hand rename re-keys the task — inputs
naming the old id dangle, or two files claim one id.

## Attractor

The wave DAG quietly re-derives over broken edges and execution orders drift
from what was reviewed.

## Resolution

Held by the two nets already in the design: a dangling `from` is a verify error
(unknown producer — the gate the start stands on), and a duplicate id is a load
error naming both files. A rename is repairable text, its damage loud and
located — the JSON form failed the same mistake as one opaque parse error.
