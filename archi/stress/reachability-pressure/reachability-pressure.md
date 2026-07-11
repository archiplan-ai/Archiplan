---
version: v0013
closed: v0013
---

# Reachability pressure

A baseline is a commit SHA: `version save` and `version anchor --repo` record `rev-parse HEAD`,
and `link audit` diffs each mapped member against it. The claim that opened this round, reported
back from a live rebase: "baselines are SHAs, not refs — a SHA stays valid after any branch
switch, nothing dangles." This round presses that claim at the seam it glosses. A SHA is a *name*,
not a *hold*: git keeps commits alive by reachability from refs, and archi records the string while
holding no ref of its own, so it exerts no reachability pressure on a member's object database. The
round asks where the recorded floor stays resolvable, and where a member is free to collect it out
from under the audit — and, when the floor is gone, whether the scan fails loud and local, the way
`scans-see-every-mapped-member` promises, or takes every member down with it.
