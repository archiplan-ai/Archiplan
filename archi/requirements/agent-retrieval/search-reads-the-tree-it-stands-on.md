---
kind: non-functional
origin: stressor(a-stale-index-lies)
satisfied-by: [Search]
deferred:
---

# Search reads the tree it stands on

Retrieval keeps no persisted derivative of the corpus: no index file, no cache, no
sidecar store. Every query walks the live doc tree and the freshly compiled model, so
a text edit — by any writer, through any editor — is searchable in the immediately
following call, and a search never writes anything anywhere.

## System Context

Every write in archi is a bare text edit into the tree; there is no daemon and no save
hook to invalidate a derivative. `source-is-the-only-truth` governs the model; this
requirement extends the same stance to retrieval: the only store search consults is
the one the rest of the tool already reads, so search can never disagree with `check`,
`query` or the files themselves.

## Satisfy

`Search` (builds its corpus per query from the doc tree and the compiled model handed
to it; holds no state between calls and owns no files in the tree).

- test — an edit to a requirement file is reflected by the immediately following search, no save between
- test — a search leaves the tree byte-identical: no files created or modified anywhere under the project
