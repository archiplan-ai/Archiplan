---
kind: functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# An intent is a problem statement

An intent anchors a requirements area: one folder per problem, the folder-named file
holding an H1 and the problem statement in the stakeholder's own terms, summary paragraph
first. The schema is the minimal one — no frontmatter, an intent has no machine fields —
and intents are flat: they do not nest, and they never appear mid-stress-session, because
requirements added during a round answer pressure while a new problem statement is a
conversation with a stakeholder. Intent files compile under the shared doc catalog:
schema, slug uniqueness, placement.

## System Context

Everything in the requirement tree hangs off some intent — placement checks read the
folder structure as meaning (`origin-records-why-placement-records-where`) — so the
anchor document itself must be schema-checked or the hierarchy has an unchecked root.

## Satisfy

`DocsCompiler` (parses the folder-named file as the intent, walks its folder as its
requirement tree, and rejects requirements that sit outside any intent folder).

- test — docs::placement_is_meaning
- test — docs::the_worked_tree_checks_out
