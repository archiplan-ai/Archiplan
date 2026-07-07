---
affects: [Compiler.Resolver]
outcome: breaking
---

# Module renamed before its types

A module holding conn edges is renamed — `operator.arch` back to `agent.arch`, a refactor with no
semantic intent — and now sorts before `conns.arch`, the module defining the conn types its edges
instantiate. The source format promises file organization carries no meaning.

## Attractor

Reproduced on v0003's compiler: the resolver collects defs and binds uses in one sorted-module
walk, so the edge looks `invoke` up before its lanes exist, exact-lane inference silently
degrades, and the un-carried statement reaches the engine. The author gets the engine's
JSON-shaped `E_CARRIER_REQUIRED` ("every instantiation names `carrier`; expected
{\"node\":\"Command\"}") instead of the compiler's targeted hint — steered toward tagging
carriers by hand, away from the real cause. The model tree itself carries the workaround as a
file named to sort late, and the determinism suite cannot see any of it: it permutes discovery
order, which the sorted walk normalizes, and never module names.

## Resolution

Broke, as filed. Answered this round by phasing the resolver: `Resolver.Defs` sweeps every
module's rel/conn/view definitions into the def table before `Resolver.Uses` binds any edge,
application or carrier against it (`Uses.bind recall(DefTable) Defs.collect`) — binding becomes a
function of the model's complete def set, never of walk position. Derived: uses-see-every-def.
