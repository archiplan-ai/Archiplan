---
kind: functional
origin: intent
satisfied-by: [Seats.Mint, Seats.Guard]
deferred:
---

# Worktrees mint on demand

A verb serving unbound work under the seat discipline mints the workspace: the branch —
created, or attached if it already exists — the worktree, the registry entry, and a printed
path. The CLI never changes the caller's directory; switching into the printed path is the
caller's move. Minting is the last resort: while seats stand, the refusal lists them —
continuation belongs to the seat already carrying the unit.

## System Context

Cli mints and records the binding (the-registry-binds-the-worktree); the Agent walks — a child
process cannot move its parent's cwd, so the printed path is the whole handshake. Attaching to
an existing branch instead of forking a duplicate is what makes pushed work resumable on any
machine (one-plan-one-worktree).

## Satisfy

`Seats.Mint` creates the branch or attaches the existing one, adds the sibling worktree,
writes the row last and prints the path — the CLI never changes the caller's directory;
`Seats.Guard`'s refusal lists standing seats first and mints only work nothing carries.

- test — the guard mints for an unbound checkout and names the seat; standing seats list instead of minting over (`the_guard_mints_for_an_unbound_checkout_and_names_the_worktree`)
- test — a pushed branch re-attaches instead of forking a duplicate (`an_unbound_checkout_mints_the_worktree_and_the_work_proceeds`)
