---
affects: [Scaffold, Cli]
outcome: surviving
---

# A nested init shadows the root

`archi init services/billing` inside a repository that is already an archiplan
project mints a second root. From that directory down, every verb's
nearest-manifest walk now resolves to the child: `check` reads a different model
depending on the shell's working directory, and nobody typed anything wrong.

## Attractor

Nearest-wins discovery is what makes monorepos of several archiplan projects work
at all, and init cannot tell a deliberate second project from a lost operator —
both look like a directory under someone else's root.

## Resolution

Holds — nesting is the monorepo shape, so init proceeds; it names what it did to
the discovery landscape: the report notes the enclosing root the new project now
shadows, so the deliberate case reads as confirmation and the lost case as the
warning it needed.
