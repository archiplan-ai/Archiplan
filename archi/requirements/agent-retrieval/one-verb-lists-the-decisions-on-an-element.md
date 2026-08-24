---
kind: functional
origin: intent
satisfied-by: [Cli, DocsCompiler]
deferred:
---

# One verb lists the decisions on an element

`archi decision ls [--links <name>] [--json]` prints one row per standing decision: slug,
`prefer → over`, and the first phrase of its rationale. `--links` narrows to the decisions
whose `links` field names the given element or doc slug; a name that resolves as neither
refuses, naming it. `--json` carries the same rows, as every `ls` here does.

## System Context

Decisions carry a checked `links:` field and had no read over it: the only paths were
lexical search and opening fourteen files. The same hole stood for requirements until
`req ls --satisfies`, and this is the same repair for the same shape — the explain chain
asks "which trades priced this element" and deserves an exact answer, not a ranked guess.

## Satisfy

`Cli` (the verb, its flag and its refusal). `DocsCompiler` (serves the standing decision
set the listing reads).

- test — `decision ls` prints one row per decision file, and the count matches the files
  on disk
- test — `--links <element>` narrows to the decisions naming it, and `--links <slug>` does
  the same for a doc slug
- test — `--links` with a name that resolves as neither element nor record refuses,
  naming it
- test — `--json` carries the same rows as the render
