---
affects: [Scaffold, SourceTree, Cli]
outcome: breaking
---

# A half init strands the tree

Init writes the manifest first, then dies — disk full, ctrl-C, a crash. The
directory now holds `archi.toml` over nothing: every verb from here finds a
project root, tries to compile it, and fails with "source directory does not
exist" — the operator's first contact with archiplan is a broken project the tool
itself made.

## Attractor

Multi-file emission has an order whether or not anyone chose one, and the manifest
is the natural first line of a naive emitter — the smallest file, the project's
"header". But the manifest is also the marker every other verb keys on: the moment
it lands, the whole tree behind it is load-bearing.

## Resolution

Order became the contract: the manifest lands last, so a project root only ever
appears over a tree that is already whole — an interrupted init leaves files no
verb yet looks at, and the create-only re-run finishes exactly the artifacts still
missing.
Answered by `init-changes-nothing-twice`.
