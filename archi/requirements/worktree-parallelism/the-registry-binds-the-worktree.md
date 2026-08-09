---
kind: functional
origin: intent
satisfied-by: [Seats.Registry]
deferred:
---

# The registry binds the worktree

A machine-local registry under the shared git directory maps each worktree to the work it
carries: branch, spec effort, plan. The file appears at archi's first touch in a repository —
no init step. Every mutating verb resolves its binding through the registry; entries reconcile
against `git worktree list` on every read, and the file never enters git history.

## System Context

The common git dir is the one place every worktree of a repository shares — visible from all
trees, tracked by none, gone with the machine. Cli reads the registry under the hood
(context-follows-the-checkout), writes it at mint (worktrees-mint-on-demand), and closes
rows whose paths git no longer lists (nothing-leaves-the-registry). The binding is the ownership truth
one-plan-one-worktree enforces and the mutation license mutation-needs-a-seat checks.

## Satisfy

`Seats.Registry` (rows under the shared git dir, born at archi's first touch; every read
reconciles against `git worktree list` and closes rows git no longer backs; the file
never enters history).

- test — a hand-removed worktree closes its row on the next read (`a_hand_removed_worktree_closes_its_row`)
