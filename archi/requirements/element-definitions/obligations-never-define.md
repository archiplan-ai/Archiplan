---
kind: functional
origin: stressor(a-semicolon-splices-past-the-gate, a-bare-modal-needs-no-splice, the-envelope-smuggles-prose)
satisfied-by: [Compiler.Definitions, Engine]
deferred:
---

# Obligations never define

No obligation reaches a stored definition through any door: the modal words — must, should,
shall, ensures, handles — reject wherever they stand in the prose, splice or no splice,
whatever the punctuation, matched case-insensitively and as whole words only. The rule is one
shared validator that the source attach pass, the statement schema and the engine's define
path all consult, so a definition the parser would reject can never be stored, rendered or
archived.

## System Context

Prose-pressure showed the comma-splice rule is a token gate with synonyms — semicolon,
em-dash, colon — that a bare modal clause needs no splice at all, and that the statement API
is a second door with no gate, while the archive's reconstruct-and-compile path assumes every
stored definition re-parses. The vocabulary is the honest detection surface; the punctuation
never was.

## Satisfy

`Compiler.Definitions` (its attach-time validation applies the modal rule to the whole
normalized text, comma or none). `Engine` (its define path consults the same validator, so
statement-API writes and replayed dumps meet exactly the gate the source path meets, and the
strict statement schema rejects early where it can).

- test — validate: each modal word rejects with no comma present; semicolon, em-dash and colon splices reject the same way
- test — validate: the words match case-insensitively and as whole words — `mustard` and `handler` pass
- test — engine: obligation prose in a definition rejects through the statement API and through direct execute alike
- test — render: no sequence of accepted statements produces a canonical render the compiler rejects
