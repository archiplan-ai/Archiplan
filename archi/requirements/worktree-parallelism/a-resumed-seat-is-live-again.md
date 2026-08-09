---
kind: functional
origin: stressor(the-operator-resumes-a-landed-seat)
satisfied-by: [Seats.Landing]
deferred:
---

# a resumed seat is live again

A landing record describes one head. When the seat's head has moved
past it, or the worktree holds uncommitted work, the record stops
counting: the row reads as live work, the sweep passes the folder by,
and the listing shows it as work in flight. Nothing is written to say
so — the state derives from the head and the tree on every read.

## System Context

A review that asks for a change sends the operator back into a landed
seat. Whatever the pull request says, the folder now holds newer work,
and freeing it would destroy the answer to the review.

## Satisfy

`Seats.Landing` compares the recorded head against the current one and
reads the tree before it trusts a landing record.

- test — a commit on top of a landed seat makes the sweep skip it and
  the listing call it live
- test — an uncommitted change does the same
