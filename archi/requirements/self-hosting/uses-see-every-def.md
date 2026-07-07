---
kind: non-functional
origin: stressor(module-renamed-before-its-types, def-after-use-in-one-module)
satisfied-by: [Compiler.Resolver]
deferred:
---

# Uses see every def

Reference binding is two-phased: every module's rel, conn and view definitions are collected into
the def table before any edge, application or carrier binds against it. What a use resolves to is
a function of the model's complete def set — never of module naming, file order, or the textual
position of a def relative to its uses. When a carrier lane genuinely cannot be inferred, the
diagnostic is the compiler's targeted hint at the edge's span, not a downstream engine error; a
conn with no source def (the preset's) keeps its def-less binding path unchanged.

## System Context

Modules compile in sorted-name order and the source format promises that file organization
carries no evaluation semantics — renaming a file must never change what compiles. Carrier
inference reads conn lane patterns out of the def table at the moment an edge binds.

## Satisfy

`Compiler.Resolver` phases the walk: its `Defs` child sweeps rel/conn/view definitions across all
modules and positions, then its `Uses` child binds edges, apps and carriers against the completed
table — `Uses.bind recall(DefTable) Defs.collect`. (`satisfied-by` pins the node;
`issues/satisfied-by-cannot-name-ports-or-edges.md`.)

- test — resolve: a conn defined in module `z`, instantiated with inferred carriers in module `a`, binds with the compiler's hint firing only when a lane is genuinely uninferable
- test — resolve: a def textually after its first use inside one module still binds
- test — resolve: an edge on a def-less conn resolves identically before and after the split
- test — source_e2e: compilation is invariant under module renaming — same content, shuffled module names, bit-identical batch
