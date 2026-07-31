---
affects: [Members, Seats]
outcome: breaking
---

# a mapping outlives its worktree

Map a member to a side worktree — an agent's scratch checkout, another
seat, any linked worktree. Let the session end. The folder and the row
stay. Months later a mint resolves the member through that row.

## Attractor

The new seat branch is born from a dead feature branch and carries its
commits from the first second. The operator sees foreign work inside a
fresh seat and cannot tell where it came from — trust in the seat
model erodes toward "always diff everything by hand".

## Resolution

Two gates close both ends of the hole. At write time, `repo map`
refuses a path that is a linked worktree and names the repo's main
checkout instead. At use time, the mint refuses a member whose
resolved checkout is a linked worktree — the refusal calls the row
stale and gives the exact `repo map` repair — while a seat's own
member worktrees (already in the binding) stay legal. Derived
`a-mapping-names-the-main-checkout` and
`the-cascade-refuses-a-side-worktree-base`.
