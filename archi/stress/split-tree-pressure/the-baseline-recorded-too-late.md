---
affects: [Archive, Members]
outcome: breaking
---

# The baseline recorded too late

The spec saves a version while a member sits dirty mid-hotfix, so no baseline lands. The team
implements the round, commits the member, and only then anchors it — the baseline now points
*after* the very delta the audit should have covered.

## Attractor

The audit's window silently opens at the wrong edge: everything between the version and the late
anchor is unaccounted by construction yet reported as clean coverage. Post-hoc anchoring — the
recovery this tool itself teaches — becomes the laundering step, and nobody chose to lie.

## Resolution

A baseline remembers how it was born. Save-time and post-hoc baselines are distinct in the
version entry, and the audit says per member when its window opened at an anchor instead of the
save — unaudited spans are named, never absorbed. Derived: `a-late-baseline-says-so`.
