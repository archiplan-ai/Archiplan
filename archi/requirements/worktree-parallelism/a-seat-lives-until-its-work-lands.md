---
kind: functional
origin: stressor(the-sideways-landing-retires-before-the-work-lands)
satisfied-by: [Seats.Landing, Seats.Registry]
deferred:
---

# a seat lives until its work lands

Retirement follows integration, never the push. A local merge puts the
work in the receiving branch at that moment, so the seat retires there
and then. A sideways landing (`--to`) and every member push do not:
they record where the work went — the branch, the receiving branch and
the head it landed at — and the seat keeps standing. The row closes
when the spec side and every member carry their work in their receiving
branches.

## System Context

A protected receiving branch leaves only the sideways path, so on a
normal team every landing is a pull request that merges hours or days
later. The window between pushing and merging is exactly where a review
asks for changes, so the workspace must survive it.

## Satisfy

`Seats.Landing` writes the landing record instead of retiring on the
sideways path and on member pushes; `Seats.Registry` holds the record
and closes the row when every side is integrated.

- test — a `--to` landing keeps the worktree and the row, and the
  listing names the branch it waits on
- test — a member push keeps its member worktree; the row closes only
  after the member's base carries the work
- test — a local merge still retires in the same move
