---
kind: functional
origin: intent
satisfied-by: []
deferred:
---

# The registry moves by verbs

The registry is operated only through the CLI, never by hand: a verb lists every worktree with
its binding, a verb drops a stale entry; mint writes entries (worktrees-mint-on-demand), merge
clears them (merge-retires-the-worktree).

## System Context

Same ground rule as every lifecycle store: files are the truth, verbs are the only writers.
The listing is the operator's view over parallel work in flight; the drop verb is the manual
repair for what self-healing against `git worktree list` cannot decide
(the-registry-binds-the-worktree).

## Satisfy
