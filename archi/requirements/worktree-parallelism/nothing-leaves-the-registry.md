---
kind: functional
origin: stressor(the-sideways-landing-retires-before-the-work-lands)
satisfied-by: [Seats.Registry]
deferred:
---

# nothing leaves the registry

No command deletes a row. A row carries `active` or `closed`: the
landing closes it, `archi worktree close` closes it by the operator's
word, and self-heal closes a row whose worktree git no longer lists.
The folder is a disposable derivative; the row is the record of what
this machine carried, and it stays. A mint whose slug names a closed
row re-opens that row instead of writing a second one.

## System Context

The folder is worth tens of gigabytes and rebuilds from one command.
The row is a hundred bytes and rebuilds from nothing. The old design
deleted both at once, so a cleared session found no thread to pull.

## Satisfy

`Seats.Registry` closes rows where it used to remove them — the close
command, the landing and the self-heal pass alike — and re-opens a row
on a mint of its slug.

- test — close marks the row and keeps it; the listing still shows it
- test — a worktree removed by hand closes its row on the next read
- test — a mint of a closed slug re-opens the same row
