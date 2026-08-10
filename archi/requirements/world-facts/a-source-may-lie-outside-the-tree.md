---
kind: functional
origin: stressor(the-source-lives-outside-the-tree)
satisfied-by: [WorldDoc, DocsCompiler]
deferred:
---

# A source may lie outside the tree

A `sources` entry is either a path into the tree, which resolves and errors when it does
not, or an external locator — a URI, a ticket id, a recording reference — written with a
scheme and kept verbatim. `check` resolves the first kind and never resolves the second.
An entry with no scheme is read as a path.

## System Context

The material behind a real observation is a conversation, a support thread, a session
recording or a row in another system, and none of that lives in this repository. A field
that only accepts tree paths leaves an honest author two options: transcribe memory into
a note so the checker is satisfied, or leave `sources` empty and have an observed fact
read exactly like a guess. Both destroy the one signal the field carries. The cost of
admitting external entries is that they cannot be verified — but an unverifiable pointer
to a real recording says more than a verifiable pointer to a note somebody typed to pass
the check.

## Satisfy

`WorldDoc` (the two entry forms in one list). `DocsCompiler` (resolves schemeless entries
as tree paths and errors on a miss; accepts a schemed entry verbatim and checks only that
it is well formed).

- test — a tree path that exists resolves; one that does not raises a located error
- test — a schemed entry passes with no filesystem access
- test — a malformed schemed entry raises a located error
- test — a fact mixing both forms passes
- test — a schemed entry counts as grounding for `world_ungrounded`
