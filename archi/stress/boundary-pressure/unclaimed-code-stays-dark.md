---
affects: [Links]
outcome: surviving
---

# Unclaimed code stays dark

The exclusion overreaches: a project lists `src/legacy/` to silence a noisy migration, or a glob
like `*.rs` sneaks in, and real code motion stops surfacing.

## Attractor

The dark-delta finding is the ratchet's teeth — "a hunk since the last version claimed by no
task and no link" is the one signal that code moved without an architectural account. An
exclusion mechanism that can eat code turns the audit into a report of whatever the project
chose to see; the honest default dies by configuration.

## Resolution

Holds on v0004 and fences the fix: exclusion is opt-in, per-project, and names prose — nothing
in the mechanism distinguishes code from prose, so the *tests* pin the contract instead: an
unclaimed `.rs` hunk in the fixture stays dark with exclusions active beside it, and this
repository's own setting names only `*.md`. The built-in exclusions (`archi/`, `.arch`,
`archi.toml`) stay built-in — the project can widen the mute, never re-include the model into
the code scan.
