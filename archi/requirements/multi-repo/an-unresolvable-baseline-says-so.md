---
kind: functional
origin: stressor(the-baseline-commit-is-gone)
satisfied-by: [Links]
deferred:
---

# An unresolvable baseline says so

A baseline is a commit SHA the record holds no ref for, so a member is free to collect it — a branch
deleted after merge and gc'd, a rebase or amend, a shallow clone that never fetched it. The audit
probes each member's delta floor before it diffs: a baseline whose commit no longer resolves is a
state of its own, reported for that member and skipped, never a `git diff` failure that aborts the
whole scan.

## System Context

The floor an audit stands on can vanish while the checkout is right there, and the shallow clone —
where the baseline commit was never fetched — is the default in CI, exactly where the audit runs. A
bare SHA carries no reachability guarantee, so "recorded" cannot mean "resolvable"; the scan must
check, not assume. This is `scans-see-every-mapped-member` held one layer down: an absent object,
like an absent checkout, narrows one member's coverage and is reported, never silently shrinking or
aborting the rest.

## Satisfy

`Links` (`Grader` probes the baseline commit before the audit diff; an unresolvable floor becomes a
per-member note and that member is skipped, the scan degrading alone rather than aborting on the
`git diff` error).

- test — a member whose baseline commit was collected is audited without aborting: the report names
  its unresolvable baseline and still surfaces another member's dark delta
- test — the audit leaks no raw `git` error and exits zero when a baseline no longer resolves
