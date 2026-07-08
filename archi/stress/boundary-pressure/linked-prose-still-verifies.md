---
affects: [Links]
outcome: surviving
---

# Linked prose still verifies

The live fold already holds deliberate links into prose — `l0085 Incidence ←
requirements/cli.md`, `l0243 Cli.version-edge ← requirements/versioning.md` — asserted or riding
as evidence because an operator judged the doc load-bearing.

## Attractor

"Excluded from the scan" hardens into "banned from the layer": verify starts skipping excluded
files, repin refuses them, or the fold silently drops links whose files match a pattern — and
deliberate spec↔doc traceability dies as a side effect of muting noise.

## Resolution

Holds on v0004 and fences the fix: exclusion governs what the *scans volunteer* — audit's delta
hunks, capture's candidate minting, the missing-link candidate search — never what links may
claim. `link add` accepts excluded files, `verify` grades them by reading the pinned file
directly, and the existing prose links stay clean through the change. Pinned by a regression: a
link claiming an excluded `.md` verifies clean while the same file's hunks stay out of the
audit.
