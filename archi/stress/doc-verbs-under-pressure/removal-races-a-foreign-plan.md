---
affects: [DocMint, Seats, Planner]
outcome: surviving
---

# Removal races a foreign plan

`req rm` pre-flights ownership against its own tree; a plan in a parallel seat
owns the slug on a branch the removing seat cannot see. The removal succeeds, the
join lands a plan owning a deleted requirement.

## Attractor

The pre-flight's guarantee silently narrows from "no plan owns it" to "no plan on
this branch owns it", and nobody re-reads the fine print.

## Resolution

Held — the narrowing is the seat model itself, not a verb defect: every write-time
guarantee in archi is tree-local, and the join re-validates everything — `plan
verify` names the orphaned ownership at the receiving checkout, the archi-merge
triage prescribes the repair. The pre-flight still pays for itself in the seat
where the mistake is cheapest.
