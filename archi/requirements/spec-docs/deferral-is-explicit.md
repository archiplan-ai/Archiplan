---
kind: functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# Deferral is explicit

Some claims are acknowledged and deliberately not addressed by the current architecture.
Deferring is a recorded decision: the `deferred` field carries the reason, check reports
the info-level `deferred_requirement` finding instead of `unsatisfied_requirement`, and
deferrals expire by being seen, never by being forgotten. An unsatisfied requirement with
an empty deferral is open — undecided work the next round owes an answer — and a
satisfied requirement cannot be deferred at all: every requirement is in exactly one
declared state — satisfied, deferred with a reason, or visibly open. Unsatisfied stays a
finding rather than an error by design: the save that closes a stress round produces open
requirements, and blocking on them would block the workflow on its own output.

## System Context

`findings-never-block` holds the CLI contract this leans on; the tri-state is what makes
the finding list a worklist instead of noise. This repository's own deferrals
(`skeletons-come-from-a-verb`, `versions-stay-searchable`) are the living precedent.

## Satisfy

`DocsCompiler` (derives the tri-state from the two fields, rejects the contradictory
satisfied-and-deferred state as `E_DOC`, and emits the per-state findings).

- test — docs::the_worked_tree_checks_out
- test — check_e2e::findings_stay_advisory_and_do_not_withhold_the_read
