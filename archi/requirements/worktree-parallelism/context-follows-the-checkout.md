---
kind: functional
origin: intent
satisfied-by: [Cli, Seats.Guard, Seats.Registry]
deferred:
---

# Context follows the checkout

Every verb resolves its context from the current directory alone: the tree by manifest
walk-up, the binding by registry lookup — is this a worktree, and what does the registry hold
for it. Three outcomes, always spelled out: bound here — proceed; bound elsewhere — refuse,
naming the owning path; unbound under the discipline — refuse listing the standing seats to
continue, or mint first work and print the path to enter. No environment variable, flag or
home config replaces this resolution.

## System Context

cwd is the only input the caller controls; Cli derives the rest under the hood — the walk-up
finds the tree, git's common dir finds the registry (the-registry-binds-the-worktree).
Spelled-out refusals are the contract that lets an agent recover without guessing.

## Satisfy

`Cli` resolves the tree by manifest walk-up from cwd; `Seats.Guard` answers with the three
spelled-out outcomes from `Seats.Registry` lookups keyed by the checkout's toplevel — no
environment variable or flag replaces cwd.

- test — bound here proceeds; bound elsewhere refuses naming the owner (`an_unbound_checkout_mints_the_seat_and_the_worktree_proceeds`)
- test — unbound with standing seats lists them and refuses to mint over them (`the_guard_mints_for_an_unbound_checkout_and_names_the_seat`)
