---
kind: functional
origin: intent
satisfied-by: [Seats.Landing]
deferred:
---

# Protected branches land by PR

A branch the manifest protects never receives a local merge: the closing
verb refuses and points at the sideways landing — push the branch and open a
PR, or `--to <branch>`. Unprotected branches merge locally from their own
checkout.

## System Context

The refusal lives in the closing verb (merge-retires-the-worktree), not in
the guard: mutating inside a seat is free, landing on shared history is the
controlled move. The manifest's `protected` list carries this one meaning
alone — the seat discipline itself is unconditional and needs no switch
(mutation-needs-a-seat).

## Satisfy

`Seats.Landing` (merge consults the manifest's `protected` list — its single meaning: a
protected branch never receives a local merge; the refusal points at `--to` plus push/PR).

- test — a protected branch refuses the local merge and names the sideways landing (`a_protected_branch_never_receives_a_local_merge`)
