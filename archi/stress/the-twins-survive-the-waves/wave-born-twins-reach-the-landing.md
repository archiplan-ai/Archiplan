---
affects: [Planner]
outcome: breaking
---

# wave-born twins reach the landing

Run a plan whose wave dispatches two sub-agents that both need one
mechanism. Each writes its own copy, because the write surfaces are
disjoint. Close the waves, run the scenarios, land the unit.

## Attractor

The scenarios bless the duplicated code, the landing carries it to the
default branch, and the twins drift apart on the next change. A field
sweep of this very repository found the git plumbing written four
times.

## Resolution

The plan lifecycle gains a cleanup stage between the last wave and the
scenarios. `plan next` announces it once, like the scenarios block: a
dedicated sub-agent sweeps the unit's whole delta for twins, folds
them with zero behavior change, and the tests must stay green with no
assertion edits. The next `plan next` moves to the scenarios, so the
scenarios always bless the folded code. An empty sweep is one line and
moves on. Derived `a-cleanup-wave-precedes-the-scenarios`.
