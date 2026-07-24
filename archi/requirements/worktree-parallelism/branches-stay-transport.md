---
kind: non-functional
origin: intent
satisfied-by: []
deferred:
---

# Branches stay transport

No branch name enters a tracked artifact — not the Archive, not PlanFile, not the journal, not
stress records. Branch awareness lives only at runtime and in the machine-local registry: the
guard, the status line, the binding.

## System Context

Archi is branch-blind by design, and blindness is what lets every record merge cleanly and
travel between machines. This claim fences the new git awareness introduced by this intent so
the worktree machinery cannot leak location into shared truth.

## Satisfy
