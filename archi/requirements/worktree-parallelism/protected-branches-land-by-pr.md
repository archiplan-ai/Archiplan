---
kind: functional
origin: intent
satisfied-by: []
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
controlled move. The manifest's `protected` list carries both meanings — its
presence switches the seat discipline on (mutation-needs-a-seat), its
entries name where local landings refuse.

## Satisfy
