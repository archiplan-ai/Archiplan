---
kind: functional
origin: stressor(a-dependent-unit-forks-from-the-default-base)
satisfied-by: [Seats.Mint]
deferred:
---

# the mint names its fork

A bare `--base <branch>` names the fork point of the home branch: the
new worktree branch grows from that branch's tip, not from the invoking
checkout's HEAD. An unknown branch refuses and names the standing
branches. Every mint report prints the branch and the short commit the
home branch forked from, so a wrong base is visible in the same breath
it happens.

## System Context

The home branch always forked from the invoking checkout's HEAD, and
the report never said so. Dependent work minted from a primary checkout
silently forked from the default base. The member half of `--base`
already names branches per member. The home half completes it.

## Satisfy

`Seats.Mint` reads the bare `--base` as the home override, passes it to
the branch creation, and the report carries `from <branch> <sha7>` on
every fresh mint.

- test — worktree_e2e: a mint with `--base <feature-branch>` forks the
  home branch from that tip, the default base stays untouched, and the
  report names the branch and sha; a mint without `--base` reports the
  checkout's own branch; an unknown base refuses
