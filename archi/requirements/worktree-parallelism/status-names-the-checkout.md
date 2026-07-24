---
kind: functional
origin: intent
satisfied-by: []
deferred:
---

# Status names the checkout

One verb answers "where am I": worktree path, branch, this checkout's registry binding, the
active plan with its open wave, version state (at / dirty since), the open stress round, and
every plan on this branch with open lifecycle. It is the first verb of any working session.

## System Context

Nothing new is stored — Cli composes what it already holds: the registry entry, Archive's
current, Sessions' open round, PlanFile state. The open-plans listing is what makes handoff self-describing: a
fresh checkout of a pushed branch discovers mid-flight work without verbal instructions.

## Satisfy
