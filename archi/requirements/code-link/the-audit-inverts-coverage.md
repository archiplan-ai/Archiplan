---
kind: functional
origin: intent
satisfied-by: [Links.Grader]
deferred:
---

# The audit inverts coverage

With deltas as the input, coverage inverts from "which links exist" to "what is
unaccounted for". `link audit` reports the hunks since the delta source claimed by no
task and no link (`unaccounted_delta` — code motion with no architectural account), the
spec elements in the active plan's scope with no asserted link and no live evidence
(`unlinked_spec_ref`), and the evidence links whose confidence fell below the floor
(`decayed_evidence` — confirm or retire, `--prune` retiring in bulk). Confidence itself
accrues as tasks carrying the same spec_ref touch the same symbol and erodes as the
symbol is rewritten without reconfirmation — observed as journal events, derived at read,
never stored. The delta source is the latest version's commit provenance or an explicit
`--since`; without either the audit says so instead of guessing
(`provenance-anchors-post-hoc` names the recovery). All of it advisory, like every
finding.

## System Context

The dark-delta finding is the ratchet's teeth — `dark-deltas-are-code` draws its scan
boundary — and an audit that blocked would get deleted from CI while one that guessed
would train skimming. The aggregate view is a spec × code incidence surface, the same
shape the stress matrix wears.

## Satisfy

`Links.Grader` (the audit sweep: per-source deltas, plan-scope coverage, confidence
folds).

- test — links::audit_sweeps_scope_coverage_and_dark_deltas
- test — links::audit_scopes_unlinked_refs_from_the_active_plan
- test — links::confidence_accrues_by_touch_and_erodes_by_decay
