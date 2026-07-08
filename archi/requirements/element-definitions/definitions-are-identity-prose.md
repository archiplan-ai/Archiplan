---
kind: functional
origin: intent
satisfied-by: [Compiler.Definitions]
deferred:
---

# Definitions are identity prose

A definition is one sentence of identity prose stating what the element *is*, at most 240
characters. Multi-sentence prose rejects, and a comma-spliced clause using a modal verb —
must, should, shall, ensures, handles — rejects: obligations live in requirement docs, not in
definitions. Every rejection is a located diagnostic naming the rule it broke, and a sentence
boundary is a terminator followed by whitespace, so dotted tokens (`mod.rs`, `plan.json`)
never split a definition in two.

## System Context

The definition field is constrained identity prose, not an open text field — the pressure to
smuggle behavior into the nearest free-text slot is constant, and requirement docs are where
obligations already live, cross-checked against the model on every `check`. The constraint is
only real if the gate rejects loudly and points at the offending comment.

## Satisfy

`Compiler.Definitions` (validation runs at attach, before resolution, over the normalized
text — block lines joined, whitespace collapsed; every violation is one located diagnostic at
the offending comment naming the rule it broke, and all of a file's violations surface in one
pass).

- test — validate: an identity sentence passes; a 241-character one rejects naming the limit
- test — validate: a second sentence rejects; `.` inside a dotted token is no boundary
- test — validate: a comma splice carrying each of must, should, shall, ensures and handles rejects naming the obligation rule
- test — cli: the diagnostic carries the file, line and column of the offending comment
