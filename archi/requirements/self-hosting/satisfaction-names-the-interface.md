---
kind: functional
origin: stressor(satisfaction-launders-through-the-node)
satisfied-by: [DocsCompiler, Links]
deferred:
---

# Satisfaction names the interface

A `satisfied-by` entry and a link `SpecRef` resolve a port path (`Engine.answer`) and canonical
edge text to the exact element, not only whole node paths. A requirement about one interface point
claims that point; the reverse-lookup that seeds a plan sees the port rather than the hub around it;
and the two reference layers — requirement satisfaction and code links — accept the same vocabulary,
so an element means the same thing whether a doc or the journal names it.

## System Context

Resolution today descends node children alone, so ports and typed edges resolve nowhere in either
layer, and a port-level claim can only launder through its owning node. The link `SpecRef` parser
already accepts canonical edge surface text; extending both layers to ports closes the asymmetry the
retired issue recorded — spec↔code finer-grained than spec↔requirement for no reason.

## Satisfy

`DocsCompiler` (the `satisfied-by` cross-check resolves port paths and canonical edge text against
the model) and `Links` (the `SpecRef` parser accepts a port path, matching the edge text it already
takes).

- test — schema: `satisfied-by: [Engine.answer]` resolves and validates against a model that declares the port
- test — schema: `satisfied-by` naming canonical edge text resolves to that edge
- test — links: `link add Engine.answer <ref>` resolves the port instead of raising `E_MODEL_REF`
- test — incidence: a requirement pinned to one port counts that port's surface, not the whole owning node's
