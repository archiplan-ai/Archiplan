---
affects: [Storage, Links]
outcome: surviving
---

# History rewrite

The repository is squash-merged, rebased and cloned shallow in CI: every commit sha recorded
anywhere in the system now points at history that no longer resolves.

## Attractor

Versions reconstruct only where git history survives; link birth records dangle off orphaned
shas; the sealed archive quietly depends on a second store it does not control.

## Resolution

Holds. Both stores under pressure — the version archive and the link journal — store content,
not references: keyframes plus hash-verified patches reconstruct any version in a shallow clone,
and birth records carry spans and span-content hashes inline. Commit shas appear only as
optional provenance, recorded when the tree is clean and depended on by nothing.
