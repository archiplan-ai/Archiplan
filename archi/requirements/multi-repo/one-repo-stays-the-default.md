---
kind: functional
origin: intent
satisfied-by: [Members, Links, Archive]
deferred:
---

# One repo stays the default

A project that declares no members is today's project, byte for byte: same
manifest schema, same journal events, same anchors, same scan boundary, same
output. Multi-repo is pay-for-what-you-declare — the single-repo shape is not a
compatibility mode but the unmarked case, and nothing about it may change out
from under existing projects.

## System Context

Every existing archiplan project, this one included, is single-repo; the journal
and version archive are append-only records those projects must keep replaying.
The qualifier is optional wherever it appears — absent means the project's own
repository — so old data and old habits stay valid without a migration.

## Satisfy

`Members` (no declarations resolves to home alone), `Links` (unqualified anchors and bare scan
keys as today), `Archive` (the entry without a baseline table is the entry as written today).

- test — with no member declarations the full existing test suite passes unchanged
- test — this project's own journal and version archive replay under the new parsers
- type-level — the member is an optional field in every schema it joins
