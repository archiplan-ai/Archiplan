---
kind: functional
origin: intent
satisfied-by: [Seats.Landing]
deferred:
---

# Member branches land by push

A member's finished branch goes to its remote — pushed by the closing verb,
integrated as a PR on the forge — never merged locally into the member's
checkout. A refused push keeps the member bound and the close idempotent: a
member's retirement is its push.

## System Context

The one deliberate breach of "archi never fetches": the push happens only
inside the explicit closing verb (merge-retires-the-worktree). Squash merges
on the forge rewrite commits, so a recorded baseline may stop being an
ancestor of the member's main — the auto base then degrades to the `--base`
question by design (the-cascade-follows-the-plan).

## Satisfy

`Seats.Landing` (push_members: the closing verb pushes each member's branch to its remote
and retires the member seat; a refused push keeps the member bound and the re-run is
idempotent).

- test — a clean close pushes the member branch to the bare remote and retires its worktree (`the_cascade_mints_member_worktrees_and_the_seat_overlay`)
- test — a refused push keeps the member bound; repairing the remote and re-running finishes the retire (`a_refused_push_keeps_the_member_until_repaired`)
