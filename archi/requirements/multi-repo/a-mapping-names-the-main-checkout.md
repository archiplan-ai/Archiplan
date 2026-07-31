---
kind: functional
origin: stressor(a-mapping-outlives-its-worktree)
satisfied-by: [Members]
deferred:
---

# a mapping names the main checkout

`archi repo map` refuses a path that is a linked worktree of the repo.
The refusal says which branch stands there, warns that a mapping
outlives the worktree, and names the repo's main checkout as the path
to map. The overlay file stays plain text — a hand edit remains the
escape for a deliberate exotic layout.

## System Context

A linked worktree answers `git status` like any checkout — nothing at
read time marks it as temporary. The one write path is the verb, so the
verb is where the hygiene lives: `git rev-parse --git-dir` differs from
`--git-common-dir` exactly in a linked worktree, and `git worktree
list` names the main checkout to point at.

## Satisfy

`Members` gains the detection at the `repo map` write: resolve the
path, compare the two git dirs, refuse with the main checkout named.

- test — multi_repo_e2e: mapping a member to a linked worktree refuses,
  the message names the standing branch and the main checkout; mapping
  the main checkout succeeds
