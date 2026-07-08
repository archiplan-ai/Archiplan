---
kind: functional
origin: intent
satisfied-by: [Compiler.Lower, Engine, Renderer]
deferred:
---

# Definitions are semantic

A definition is model, not trivia: it joins its element's identity — a restatement with a
divergent definition rejects exactly as divergent ports do, while an omitted one makes no
claim — it rides the lowered statements agents read, and it survives the archive: the
canonical render emits definitions in place and re-compiling that render reproduces the same
model and the same bytes.

## System Context

Source is the only truth and versions hash the canonical render — a definition living outside
the render would evaporate on the first save/reconstruct cycle, and one living outside the
statement batch would be invisible to every agent that reads the model through the envelope.
Definitions therefore enter the same idempotent `define` statements everything else enters
through, and archived versions minted before definitions existed reconstruct unchanged.

## Satisfy

`Compiler.Lower` (the lowered `define` statements carry each element's definition, so the
batch agents read holds them). `Engine` (stores the definition on its element, no-ops an
identical restatement, rejects a divergent one with the stored actual — and definition prose
entering through the statement API meets the same shared validator the source path uses).
`Renderer` (the canonical render emits each definition as the trailing comment of its defining
line, so saves, seals and reconstruction carry them unchanged).

- test — engine: an identical restatement no-ops; a divergent definition rejects like divergent ports; an omitted one makes no claim
- test — statements: `define` statements carry definitions through the JSON surface under the strict schema, and invalid prose rejects there with the same rule
- test — render: a model with definitions renders canonically, recompiles to the identical model and re-renders byte-for-byte
- test — versions: a save after definitions land mints, and the archived version reconstructs against its seal
