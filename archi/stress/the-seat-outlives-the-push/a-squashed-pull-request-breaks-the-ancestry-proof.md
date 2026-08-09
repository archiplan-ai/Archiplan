---
affects: [Seats.Landing]
outcome: breaking
---

# a squashed pull request breaks the ancestry proof

Merge the pull request with the forge's squash button, the default on
GitHub. The branch tip never becomes an ancestor of the receiving
branch: the content landed under a new commit with a new sha.

## Attractor

An ancestry test answers "not integrated" forever. Seats pile up on
every squash-merging team, and the sweep that was supposed to free the
disk frees nothing.

## Resolution

Integration is proven by content, not by lineage: the work is in when
the branch carries nothing the receiving branch lacks. The ancestry
test stays as the cheap first pass, and the content test answers the
squash. Derived `integration-is-proven-by-content`.
