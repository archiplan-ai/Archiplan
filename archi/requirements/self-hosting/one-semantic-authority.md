---
kind: non-functional
origin: intent
satisfied-by: [Engine]
deferred:
---

# One semantic authority

Shapes, port discipline, scope rules and idempotence are checked in exactly one place. The
compiler adds name resolution, modularity and source locations — it never re-implements a
semantic rule, so surface and statement layer cannot drift apart.

## System Context

Two front doors exist (surface source and JSON statements); a rule enforced in only one of them
is a rule that does not exist.

## Satisfy

`Engine` executes every batch — compiled or hand-built — atomically: a statement applies, no-ops
on exact restatement, or rejects with a structured error, and any rejection rolls the whole
batch back.

- test — errors::batches_are_atomic, errors::a_failed_statement_leaves_no_partial_state
- type-level — the compiler's only output is Statement values; every write passes Workspace::execute
