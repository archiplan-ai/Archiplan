---
kind: functional
origin: intent
satisfied-by: [Planner]
deferred:
---

# Verify is the worklist

`plan verify` reports the whole authoring worklist in one read: structural
breaks, unowned candidates, verifications keyed to unowned requirements or
missing on owned ones, empty task descriptions, tasks with no declared
outputs (advisory — capture cannot attribute their delta), and a summary
whose stack mapping does not close both ways. Errors gate `plan start` and
`plan next`; notes advise.

## System Context

The report is the plan stage's only feedback surface — a check that
enforces at the gate but stays silent in verify hides the worklist until
the worst moment. One validator serves verify, start and next
(verifications-gate-the-start).

## Satisfy

`Planner` (one validator; errors gate the lifecycle, notes advise).

- test — plans::verify_flags_structure_and_notes_drift
- test — plans::curation_selects_from_matched_and_scopes_the_verification_duty
