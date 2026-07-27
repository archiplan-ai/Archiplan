---
kind: functional
origin: intent
satisfied-by: [Planner]
deferred:
---

# Matches are candidates

The derived matched set is the candidate list, not the assignment: each
task's authored `owns` selects the requirements it answers for — a strict
subset, at least one when candidates exist, never a slug the lookup did not
match. Several tasks may touch one element without all of them answering
for its requirements.

## System Context

Ownership restores curation without giving back retyping: the candidates
stay derived and ever-fresh (tasks-derive-never-retype), the selection is
the one authored judgment planning adds. Verification duty follows
ownership (verifications-gate-the-start), which removes the over-gating
where one requirement demanded proofs from every task touching its
element. `plan show` marks unowned candidates so the plan stage sees what
is still uncurated.

## Satisfy

`Planner` (candidate derivation, ownership validation, the owned flag on
the derived view).

- test — plans::curation_selects_from_matched_and_scopes_the_verification_duty
