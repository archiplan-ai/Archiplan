---
kind: functional
origin: stressor(the-baseline-recorded-too-late)
satisfied-by: [Archive]
deferred:
---

# A late baseline says so

A baseline records how it was born: at save on a clean tree, or anchored post hoc. The version
entry keeps the two distinct, and the audit's per-member report names an anchored baseline as
the late edge of its window — the span between the version and a late anchor is reported
unaudited, never absorbed into clean coverage.

## System Context

Post-hoc anchoring is the recovery the tool itself teaches for the dirty-at-save case, and a
baseline recorded after the delta it should have covered under-reports by construction. The
record cannot always be complete; it can always refuse to claim completeness it does not have.

## Satisfy

`Archive` (save-born and anchor-born baselines are distinct in the entry; the audit reads the
distinction and words its window accordingly).

- test — a baseline recorded by `anchor --repo` carries its post-hoc birth in the version entry
- test — the audit under a post-hoc baseline names the unaudited span for that member
