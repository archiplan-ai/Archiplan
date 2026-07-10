---
kind: functional
origin: intent
satisfied-by: [Compiler.Resolver]
deferred:
---

# Names resolve from the inside out

The lexical rule for a path's first segment: the innermost enclosing block's node's
children — semantic children, wherever in the project they are defined — then enclosing
blocks outward, then file scope: the module's own top-level definitions, its imports and
the preset. Later segments descend. Block children therefore shadow file-scope names, and
preset names are ambient everywhere — visible without import, shadowable by children.
Rel, conn and view names resolve in their flat namespaces against file scope (own ∪
imported ∪ preset), and an edge statement's kind follows its type name: a rel name reads
its ends as whole node paths, a conn name splits each end into node and port.

Every reference lowers to an absolute path — the statement layer keeps its
no-ambient-scope contract, so nothing downstream of the compiler ever resolves a name.

## System Context

`open` blocks make containment semantic rather than textual (`open-reopens-a-scope`), so
"the block's children" must mean the node's children across all modules, not the lines
above. The preset arrives through the manifest (`the-manifest-marks-the-root`), and the
absolute-path addressing the resolver lowers onto is the contract agents read through
(`agents-read-lowered-statements`).

## Satisfy

`Compiler.Resolver` (resolves each path's first segment against the semantic tree from
the innermost block outward, then the module's file scope; descends later segments; emits
absolute paths only).

- test — resolve::block_children_shadow_file_scope
- test — resolve::cross_file_defs_opens_and_flows_resolve
- test — semantics::references_are_absolute_only
- test — source_e2e::the_auth_fixture_compiles_and_answers_queries
