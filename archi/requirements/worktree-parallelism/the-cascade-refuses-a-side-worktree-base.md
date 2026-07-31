---
kind: functional
origin: stressor(a-mapping-outlives-its-worktree)
satisfied-by: [Seats.Mint]
deferred:
---

# the cascade refuses a side-worktree base

When a cascaded member resolves to a linked worktree of its repo, the
mint refuses. The refusal calls the mapping row stale, names the branch
standing there, and gives both repairs verbatim: `archi repo map
<member> <main-checkout>` or `--base <member>=<branch>`. An explicit
`--base` for that member lifts the gate — the branch is named, the
checkout's identity stops mattering. A member already in the seat's
binding is the seat's own worktree and is never gated.

## System Context

The mint trusts the address book; a stale row silently turns "branch
from the member's checkout" into "branch from a dead feature line".
The gate moves that failure from the merge diff — days later, foreign
commits inside a fresh seat — to the mint, where the repair is one
command. The new member worktree also anchors its folder on the repo's
main checkout, never beside the mapped path, so no seat nests inside
scratch folders.

## Satisfy

`Seats.Mint` gates the auto-base arm of the cascade: resolve the
member, detect the linked worktree, refuse batched with the other
member refusals; `--base` for the member skips the check; placement
derives from the main checkout's toplevel.

- test — worktree_e2e: a member mapped to a linked worktree refuses the
  mint naming `repo map` and `--base`; the same mint with `--base
  <member>=<branch>` proceeds and the member worktree lands beside the
  main checkout; a seat extension over an existing member stays silent
