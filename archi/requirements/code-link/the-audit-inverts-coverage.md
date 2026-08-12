---
kind: functional
origin: intent
satisfied-by: [Links.Grader]
deferred:
---

# The audit inverts coverage

With deltas as the input, coverage inverts from "which links exist" to "what is
unaccounted for". `link audit` reports the hunks since the delta source claimed by no
task and no link (`unaccounted_delta` — code motion with no architectural account) and the
spec elements in the active plan's scope that no link answers (`unlinked_spec_ref`). Those
two are the whole report: a link stands or it is retired, so there is no third grade to
sweep for (`a-link-stands-asserted-or-it-does-not-stand`). The delta source is the latest
version's commit provenance or an explicit
`--since`; without either the audit says so instead of guessing
(`provenance-anchors-post-hoc` names the recovery). All of it advisory, like every
finding.

## System Context

The dark-delta finding is the ratchet's teeth — `dark-deltas-are-code` draws its scan
boundary — and an audit that blocked would get deleted from CI while one that guessed
would train skimming. The aggregate view is a spec × code incidence surface, the same
shape the stress matrix wears.

## Satisfy

`Links.Grader` (the audit sweep: per-source deltas and plan-scope coverage).

- test — links::audit_sweeps_scope_coverage_and_dark_deltas
- test — links::audit_scopes_unlinked_refs_from_the_active_plan
