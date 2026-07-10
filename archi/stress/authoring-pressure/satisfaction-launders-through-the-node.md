---
affects: [DocsCompiler, Links]
outcome: breaking
---

# Satisfaction launders through the node

A requirement's `satisfied-by` resolves through node descent alone — the same shape the link layer
uses to resolve a `SpecRef`. A claim that honestly pins an interface point (`Engine.answer`) or a
typed edge has nowhere to land: the port path resolves nowhere, so the author launders the claim
through the whole owning node, and a requirement about one port of a hub reads as a claim over the
entire node.

## Attractor

Over-approximation with no error: the coarse claim inflates the invariant surface incidence
measures and dulls the reverse-lookup that seeds a plan's requirements — a hub satisfying a dozen
requirements it only touches at one port each. The asymmetry is worse for being arbitrary: the link
layer already accepts canonical edge surface text as a ref but rejects port paths, so spec↔code and
spec↔requirement disagree on what an element is for no reason the author can see. Found authoring
this model; the practiced workaround was a comment pointing back at the now-retired issue.

## Resolution

Broke, as filed. Answered this round by making ports resolvable references in both layers — the
`satisfied-by` cross-check and the link `SpecRef` parser — and letting `satisfied-by` additionally
accept canonical edge text, so a claim names exactly the element it pins and the two layers accept
one vocabulary. Derived: satisfaction-names-the-interface.
