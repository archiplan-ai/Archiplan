---
affects: [Cli]
outcome: surviving
---

# Real failures stay loud

`version save` fails for real: the model does not compile, the archive is corrupt, or two
sessions sit open at once. Scripts and CI read the exit code.

## Attractor

"A no-op is a success" overreaches into "save always exits 0", and pipelines lose the one bit
they need: a compile diagnostic or a jammed session state scores as a clean run, and the error
philosophy — stable codes, loud failures — erodes at the CLI edge it was written for.

## Resolution

Holds on v0004 and fences the fix: only the benign no-op — unchanged model, at most one open
session — earns exit 0. Compile diagnostics keep exit 1 before the archive is even consulted,
and two open sessions keep failing the save loudly. Pinned by a regression: two open sessions
plus an unchanged save exit 1.
