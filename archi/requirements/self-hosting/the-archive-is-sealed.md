---
kind: non-functional
origin: intent
satisfied-by: [Archive]
deferred:
---

# The archive is sealed

Every archived version reconstructs bit-for-bit from keyframes and patches and verifies against
the hash chain in the manifest — in a shallow clone, years later, with no git history. Editing a
keyframe, a patch or a manifest entry is a compile error, not a silent drift.

## System Context

Git history is provenance and the recovery path, never a dependency: squash merges and shallow
CI clones must not orphan the record.

## Satisfy

`Archive` stores keyframes plus forward patches under an append-only manifest; reconstruction
verifies the sealed hash, and check re-verifies the whole chain — dense ids, linear parents, no
stray files — on every run.

- test — flip one byte in a stored patch; archi check fails with E_ARCHIVE naming the version
- test — read_e2e::query_composes_filters_and_reads_sealed_versions reads a query at a reconstructed version
