---
kind: functional
origin: stressor(the-sideways-landing-retires-before-the-work-lands)
satisfied-by: [Cli, Seats.Registry]
deferred:
---

# the listing carries the status

`archi worktree ls` prints the state of every row and of every member
under it: live work, waiting on a named branch, or closed. A waiting
line says which branch it compared against, so a stale local branch
reads as "not here yet" and not as a verdict about the forge.
`--status active|closed|all` narrows the read and composes with the
existing filters. `archi status` in an unbound checkout names the rows
that stand, so a checkout that carries nothing still points at the work.

## System Context

A registry that keeps its rows is a record, and a record needs a view.
The incident that opened this round was invisible precisely because no
listing showed that the code had landed while the spec had not.

## Satisfy

`Cli.worktree` renders the per-row and per-member state and accepts
`--status`; `Seats.Registry` serves rows unfiltered and the surface
narrows.

- test — ls shows a waiting row with its branch, a live row and a
  closed row; `--status` keeps each in turn and an unknown value
  refuses naming the three
- test — status in an unbound checkout names the standing rows
