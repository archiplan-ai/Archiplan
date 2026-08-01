---
kind: functional
origin: intent
satisfied-by: [Seats.Landing]
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

`Seats.Landing` (merge into the current branch or `--to` a new one; plan_gate refuses while
the seat's plan is open; retire removes the worktree, scrubs seat artifacts and clears the
registry row in the same move).

- test — a clean merge lands the work and retires the seat in one verb (`a_clean_merge_lands_the_work_and_retires_the_worktree`)
- test — an open plan refuses the landing until `plan close` (`a_worktree_lands_only_after_its_plan_closes`)
- test — `--to` lands the worktree head on a new branch without touching the current one (`to_lands_the_worktree_head_on_a_new_branch_without_merging`)
- test — a conflicted merge stops, keeps the seat and names the ceremony (`a_conflicted_merge_stops_and_keeps_the_worktree`)
