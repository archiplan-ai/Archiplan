---
kind: non-functional
origin: intent
satisfied-by: [Seats.Registry]
deferred:
---

# Branches stay transport

No branch name enters a tracked artifact — not the Archive, not PlanFile, not the journal, not
stress records. Branch awareness lives only at runtime and in the machine-local registry: the
guard, the status line, the binding.

## System Context

Archi is branch-blind by design, and blindness is what lets every record merge cleanly and
travel between machines. This claim fences the new git awareness introduced by this intent so
the worktree machinery cannot leak location into shared truth.

## Satisfy

`Seats.Registry` (the rows live under the shared git dir — visible to every worktree,
tracked by none; branch names appear only there and in runtime output, never in the
archive, the journal, plans or stress records).

- test — a registry entry never enters git history: the seat's `git add -A` commits no machine paths (`the_cascade_mints_member_worktrees_and_the_seat_overlay`)
- test — grep the archive, journal and plan of a seated project for its branch name: absent
