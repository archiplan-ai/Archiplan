---
affects: [Seats.Landing, Seats.Registry]
outcome: breaking
---

# the sideways landing retires before the work lands

A protected receiving branch leaves one path: land sideways, push, open
a pull request. The landing creates the branch and retires the seat in
the same breath — folder removed, row erased — while the work sits in a
pull request nobody has merged. Members are worse: they always go by
push, so their worktrees never survive the moment of pushing.

## Attractor

The window between "pushed" and "merged" has no workspace and no
record. A review that asks for a change finds nothing to change in. A
cleared session finds no row, no folder, and reconstructs the past from
git archaeology — landing on the wrong round.

## Resolution

The seat is retired by integration, not by the push. A sideways landing
records where it landed and keeps the seat standing; the row closes
when the spec and every member carry their work in their receiving
branches. Derived `a-seat-lives-until-its-work-lands` and
`nothing-leaves-the-registry`.
