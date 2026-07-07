---
affects: [Planner]
outcome: surviving
---

# Hand-edited lifecycle

An agent blocked at a gate edits plan.json directly — flips the state to started, erases the
scenario latches, widens closed waves — instead of satisfying the gate.

## Attractor

Lifecycle state becomes decoration: gates hold only the agents that choose to be held, and the
plan's history stops being evidence of anything.

## Resolution

Holds, with a caveat. Every verb re-reads and re-validates plan.json before acting, so a
structurally ill state surfaces as an error on the next verb rather than drifting silently; the
coverage gate recounts asserted links from the journal every time, so it cannot be edited into
passing. The caveat feeds the next round: a hand-edit that stays structurally valid — a widened
outputs list, a reworded verification — is indistinguishable from authorship, by design.
