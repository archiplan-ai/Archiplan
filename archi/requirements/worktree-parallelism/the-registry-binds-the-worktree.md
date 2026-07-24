---
kind: functional
origin: intent
satisfied-by: []
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
(context-follows-the-checkout), writes it at mint (worktrees-mint-on-demand), and drops
entries whose paths git no longer lists. The binding is the ownership truth
one-plan-one-worktree enforces and the mutation license mutation-needs-a-seat checks.

## Satisfy
