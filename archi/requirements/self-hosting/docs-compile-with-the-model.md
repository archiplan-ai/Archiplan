---
kind: non-functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# Docs compile with the model

Requirements, intents, sessions and stressors are prose with machine fields, and the build
cross-checks every machine field: a satisfied-by path that no longer resolves, a slug collision,
a misplaced origin — each breaks check at the offending file and line. Prose cannot silently rot
against the source.

## System Context

The documents live beside the model in one repository and are edited by many hands, most of them
agents.

## Satisfy

`DocsCompiler` parses every doc against its schema and validates satisfied-by against the live
model and slugs project-wide, so a model rename surfaces at the requirement that names it on the
very next check.

- test — rename a model node a requirement names; check reports E_MODEL_REF at that requirement's file
- test — duplicate a slug across primitives; check reports E_SLUG at both sites
