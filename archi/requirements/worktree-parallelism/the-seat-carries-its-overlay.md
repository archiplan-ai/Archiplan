---
kind: functional
origin: intent
satisfied-by: [Seats.Mint, Members]
deferred:
---

# The seat carries its overlay

A minted worktree resolves members through its own overlay, written at mint:
every cascaded member points at its member worktree; members outside the
cascade get no row — narrowed scope, never someone else's checkout. Seat
artifacts never enter git: minting teaches the repository to exclude them,
and a retire scrubs them before removal.

## System Context

The overlay is gitignored, so a fresh worktree is born blind to members —
baselines at save, wave capture and link verify would silently narrow to
home without this row set. The exclusion rides the repo-local exclude every
worktree shares, so an agent's `git add -A` cannot leak machine paths into a
branch (branches-stay-transport).

## Satisfy

`Seats.Mint` writes the overlay into the minted worktree — every cascaded member points at
its member worktree, members outside the cascade get no row — and teaches the repository's
shared exclude the seat artifacts; `Members` resolves through the overlay first. Retire
scrubs before removal.

- test — the seat overlay resolves members to their worktrees from inside the seat (`the_cascade_mints_member_worktrees_and_the_seat_overlay`)
