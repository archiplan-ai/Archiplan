---
kind: non-functional
origin: stressor(one-blob-of-links)
satisfied-by: [Links.Canonizer]
deferred:
---

# Hash contract is versioned

Every stored hash names the canonicalizer that produced it. When the tokenization rules change,
old pins do not silently reinterpret: verification surfaces the mismatch as its own state,
distinct from drift, and demands a rehash — an alarm about the ruler, never confused with an
alarm about the thing measured.

## System Context

The canonicalizer evolves with the tool; pins in the journal outlive any given release.

## Satisfy

`Links.Canonizer` stamps its version into every pin it mints, and grading compares the stored
stamp before comparing hashes, grading CanonicalizerMismatch ahead of Clean or Drifted.

- test — rewrite a stored pin's canonicalizer stamp; verify grades canonicalizer_mismatch and fails the asserted link
- type-level — pins carry the canonicalizer field by construction; a pin without one does not parse
