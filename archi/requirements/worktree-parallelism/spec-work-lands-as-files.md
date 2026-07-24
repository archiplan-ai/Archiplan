---
kind: non-functional
origin: intent
satisfied-by: []
deferred:
---

# Spec work lands as files

A delegated round returns paths, not payloads: every fan-out delegate
writes its findings as the files that carry them — one stressor, one
requirement, one module per file — and the orchestrator gates the round on
materialization: `archi check` plus a count of the files claimed against
the files on disk. A finding that is not a file on disk does not exist.

## System Context

The file-per-pressure format was built for parallel writers — files never
conflict where a store would — but an orchestrating harness pushes the
opposite habit: return structured data upward. The gate is the seam
(parallel-editing-discipline): content files land from any number of
delegates, while lifecycle verbs stay with one orchestrator
(one-plan-one-worktree). The skills and the scaffolded brief carry the
rule to every agent in a project.

## Satisfy
