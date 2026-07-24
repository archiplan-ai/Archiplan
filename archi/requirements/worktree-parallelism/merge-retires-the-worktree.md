---
kind: functional
origin: intent
satisfied-by: []
deferred:
---

# Merge retires the worktree

One verb closes a worktree: it merges the work into the current branch, or lands it on a new
branch — the caller chooses — then removes the worktree and clears its registry entry in the
same move. A seat lands only after its plan closes — work mid-wave never merges; spec-only
seats land freely. A protected receiving branch refuses the local path
(protected-branches-land-by-pr); members land by push (member-branches-land-by-push). No path
leaves a dangling binding: retirement is part of the merge, not a separate chore.

## System Context

Merging into an existing branch runs from the checkout holding it — git allows a merge only
where the target is checked out; landing on a new branch needs no target checkout at all. The
reconciliation that follows stays verb-driven — remint for collided saves, repin for stale
pins (pins-survive-a-remint), fold for concurrent rounds — and the verb names each step it
cannot run itself. Clearing the entry rides on the-registry-moves-by-verbs. The default unit
— spec, plan, code — rides one seat and lands once; a spec merged early serves only a
parallel dependent effort that must pin its published version.

## Satisfy
