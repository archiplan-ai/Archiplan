---
affects: [Engine, Compiler.Definitions]
outcome: breaking
---

# The envelope smuggles prose

An agent writes the model through the statement API instead of source:
`{"stmt": "define", "node": "A", "doc": "<three sentences of obligations>"}`. No comment was
ever lexed, the attach gate never ran — and the next canonical render emits prose the parser
itself would reject, so the archive is poisoned by its own reconstruct-and-compile path.

## Attractor

Two doors into one store, one gate at one door. Validation living in the comment-attach pass
makes prose legality a property of the source pipeline, while `define` statements are the
engine's own API — dumps replay through it, agents write through it, and every one of those
writes claims the same identity the source path claims.

## Resolution

Broke at the second door: a definition only a comment gate validates is a definition the
envelope can forge. One shared validator, both doors — the engine's define path consults the
same rule the attach pass consults, so no stored definition can exist that the parser would
refuse to read back. Answered by `obligations-never-define` (its shared-validator clause);
`definitions-are-semantic` already pins the round-trip this protects.
