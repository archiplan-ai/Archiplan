---
kind: functional
origin: intent
satisfied-by: []
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
