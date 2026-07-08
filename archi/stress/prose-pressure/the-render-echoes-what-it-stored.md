---
affects: [Renderer, Compiler.Definitions]
outcome: surviving
---

# The render echoes what it stored

A definition arrives as a three-line block with doubled spaces and a trailing tab; the
canonical render emits it as one trailing comment. If storage kept the raw text, render →
compile → render would normalize on the second pass and the bytes would drift — every seal
after the first would lie about the model it sealed.

## Attractor

Byte-stability is the archive's contract with itself; any layer that stores unnormalized text
while rendering normalized text — or the reverse — breaks the contract exactly one save later,
where it is most expensive to notice.

## Resolution

Holds by construction in the spec as written: `definitions-are-identity-prose` validates over
the normalized text — block lines joined, whitespace collapsed — so the normalized form is the
only form that exists past attach, and `definitions-are-semantic` demands the render recompile
byte-for-byte, which the stored-equals-emitted form satisfies trivially. The two claims
interlock; no new requirement.
