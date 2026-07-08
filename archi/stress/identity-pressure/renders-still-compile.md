---
affects: [Renderer, Compiler, Engine]
outcome: surviving
---

# Renders still compile

The canonical render round-trips by contract: `version show`'s output compiles as a single
module and recreates the identical model; `DocsCompiler` compiles reconstructed renders for
every pinned check; the statement batch replays through the engine.

## Attractor

The new order breaks a replay invariant: applications sort ahead of the application that
attaches their outer port and the engine refuses the batch (`NoOuterPort` — delegation chains
must read outward-in), or classifier edges sort after the shapes that consult them and typed
patterns stop matching. The render is canonical but dead: pinned checks, seeded trees and agent
replays all fail on bytes the archive calls authoritative.

## Resolution

Holds on v0004 and fences the fix: the sort respects every semantic precedence the lowering
already encodes — classifier rel edges keep landing before the shapes that consult them (surface
order applies *within* a topological rank, never across), and applications order by their
delegation chains, outer before inner, with surface order only among the ready set. Authoring
order stops being load-bearing; the engine's preconditions never were authoring-order's to
satisfy. Pinned by a regression: a delegation chain authored inner-module-first — which today
fails to compile under adverse module names — compiles under the chain-ordered lowering.
