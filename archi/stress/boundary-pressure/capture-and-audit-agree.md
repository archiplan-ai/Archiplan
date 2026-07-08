---
affects: [Links, Planner]
outcome: surviving
---

# Capture and audit agree

Two scans read the tree: capture diffs the wave's item-hash index against it when `plan next`
closes a wave, and the audit diffs the whole delta against the fold. Both decide what counts as
code.

## Attractor

The boundary forks: the exclusion lands in one scan but not the other, and a file becomes a
capture candidate the audit cannot see — or audit-dark while capture treats it as a declared
output's leftover. Candidates and coverage stop agreeing, and the operator reconciles two
definitions of "the code" by hand.

## Resolution

Holds on v0004 — both scans already share `code_files` and the same built-in exclusions — and
fences the fix: the `[audit] exclude` list is read at one seam and consulted by both walks
(`code_files` for capture, candidates and leftovers; `delta_hunks` for the audit), so one
setting moves one boundary everywhere. Pinned structurally: capture's wave scan calls the same
excluded walker the audit's coverage sweep uses.
