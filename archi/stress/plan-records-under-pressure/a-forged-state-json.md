---
affects: [PlanState, Planner]
outcome: surviving
---

# A forged state json

A hand edit stamps `state: completed` into state.json — gates skipped, the seat
lands a plan that never ran its waves.

## Attractor

Lifecycle becomes decorative: whatever the file says is believed, and the merge
gate blesses unfinished work.

## Resolution

Held at today's trust line — the same forgery is one field away in plan.json
now, and the discipline (never hand-edit lifecycle; verbs only) plus the join
review are the existing answer. The new shape narrows the forgeable surface to
one small file whose diff is impossible to miss in review. Load still validates
shape — unknown fields and states refuse — so only a deliberate, well-formed
lie passes, and no tool can stop an author lying to their own record.
