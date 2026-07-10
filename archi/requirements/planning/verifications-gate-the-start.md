---
kind: functional
origin: intent
satisfied-by: [Planner]
deferred:
---

# Verifications gate the start

A verification is authored per task per matched requirement — how the implementer will
prove the claim: a failing test, a runtime contract, a migration assertion, whatever the
requirement's prose prescribes — keyed by the requirement's slug in the task's
`verifications` map, living on the plan rather than the requirement because the proof
strategy is execution-shaped. `plan start` refuses to open the first wave until the plan
is structurally clean and every matched requirement carries at least one verification;
`plan verify` runs the same checks on demand against the current spec.

## System Context

The reverse lookup names the obligations; without a forcing function the proofs would
arrive as good intentions. Gating the start is the cheapest moment — nothing is in
flight, and the author who cut the tasks still holds the context.

## Satisfy

`Planner` (the structural and verification gates share one validator, consulted by start
and verify alike).

- test — plans::the_lifecycle_captures_gates_and_latches_scenarios
- test — plans::verify_flags_structure_and_notes_drift
