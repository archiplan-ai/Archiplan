---
affects: [Compiler.Resolver]
outcome: breaking
---

# Def after use in one module

One file, no imports involved: an edge instantiates a conn whose `def conn` sits later in the
same module — the author declared the wiring first and the vocabulary at the bottom, as prose
would.

## Attractor

Reproduced on v0003's compiler: the same silent degradation as the cross-module case, because the
single pass walks items in textual order too — the bug was never really about module *names*. Any
fix that only reorders whole modules (dependency-sorting them, compiling importers last) survives
the rename pressure and dies here; the attractor is a shallow patch that hardens the accident
into policy.

## Resolution

Broke, and pinned the fix's shape: the def sweep must complete across *all* modules and *all*
positions before the first use binds — a two-phase resolver, not a smarter walk order. Same
answer as module-renamed-before-its-types; same derived requirement (uses-see-every-def), whose
regression tests fix both textures: a conn defined in module `z` used in module `a`, and a def
textually after its first use in one file.
