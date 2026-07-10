---
kind: functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# Slugs are the reference currency

Every archi-level primitive — intent, requirement, stressor, session — is addressed by
its slug: the filename, auto-derived from the H1 name by lowercasing and collapsing runs
of non-alphanumerics to `-`. A name that does not derive to its filename is `E_SLUG`;
section-scale requirements derive theirs from the heading. Slugs are unique project-wide
across all primitive kinds — a collision reports both sites — because origins, stress
records, search cards and plan reports all speak slugs and nothing disambiguates further.
Model elements are the deliberate exception: a node's name as written is its identifier,
and the language imposes no casing convention.

## System Context

Two namespaces, one currency each: docs address each other by slug, models by absolute
path. Every cross-document reference would need a second key the moment slugs stopped
being unique, and branch-parallel authoring makes the collision a merge-time fact only
the post-merge check can catch (`parallel-editing-discipline`).

## Satisfy

`DocsCompiler` (derives, checks and de-duplicates slugs across every primitive kind in
one project-wide pass, reporting both colliding sites).

- test — md::slugs_derive_kebab
- test — docs::slugs_and_references_hold_project_wide
