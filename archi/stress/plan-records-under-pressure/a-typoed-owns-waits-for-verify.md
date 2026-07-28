---
affects: [Planner]
outcome: surviving
---

# A typoed owns waits for verify

Ownership moved from a validated verb argument to hand-edited frontmatter: a
typoed slug now sits unnoticed until verify runs.

## Attractor

The curation guarantee decays from "cannot be wrong" to "wrong until someone
checks", and execution starts on phantom ownership.

## Resolution

Held by the gate that already exists: `plan start` refuses on verify errors, and
an owns entry outside the matched set is one — the typo cannot reach execution.
The window between edit and verify is the same one satisfied-by lives in, and
the workflow's check-after-every-round habit keeps it short.
