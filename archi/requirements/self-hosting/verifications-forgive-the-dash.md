---
kind: functional
origin: stressor(the-hyphen-voids-the-verification)
satisfied-by: [DocsCompiler]
deferred:
---

# Verifications forgive the dash

A `Satisfy` bullet that opens with `test` or `type-level` is recognized as a verification whatever
separator follows — ASCII hyphen, en-dash, or em-dash all count the same — and a bullet that names
a verification tag yet still misses the shape draws a located diagnostic that prints the canonical
form, never a silent pass into prose. A would-be verification can degrade a requirement's score
only over the author's explicit choice, never by a punctuation slip.

## System Context

The doc schema is strict by design — every field present, sections ordered — but strictness that
fails *silently* trains the author to distrust the finding, not the file. The recognizer sits in
`DocsCompiler`'s parse-and-schema pass, the same place that already emits located diagnostics for
every other doc deviation, so the targeted hint costs nothing new to surface.

## Satisfy

`DocsCompiler` (its verification-bullet recognizer accepts `-`/`–`/`—` after the tag; a bullet
opening with a verification tag that still fails the shape raises a located diagnostic naming the
canonical form instead of counting zero in silence).

- test — schema: a `Satisfy` bullet written with an ASCII hyphen, an en-dash, and an em-dash each count as one verification
- test — schema: a bullet opening `test`/`type-level` that misses the shape raises the located diagnostic naming `- test — …`, not a silent pass
- test — schema: a `*`-bulleted line or a mistyped tag stays ordinary prose and the requirement's other verifications are unaffected
