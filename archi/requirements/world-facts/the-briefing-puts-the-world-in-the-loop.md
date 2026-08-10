---
kind: functional
origin: stressor(nothing-teaches-the-wing)
satisfied-by: [Scaffold, AgentBrief]
deferred:
---

# The briefing puts the world in the loop

The briefing archi installs carries the wing: the `world` verb, the shape of a fact, the
rule that a fact names no element of the model, and a step in the spec loop where
conditions are captured before requirements are derived. `sync-skills` reports the briefing
as changed on every project that upgrades.

## System Context

Every mechanism the wing grew over six rounds is invoked by a procedure, and the procedure
is the briefing. It describes a chain from intent to requirements to model that was written
before the wing existed, so an agent following it faithfully produces a spec with an empty
wing and never notices the omission. An empty wing is indistinguishable from a project
whose author had nothing to say about the world, so the failure is silent and permanent.
This is the one requirement whose absence makes every other one in this intent inert.

## Satisfy

`Scaffold` (installs the briefing text that names the verb, the shape and the loop step).
`AgentBrief` (the durable carrier: the workflow skill and the CLAUDE.md block).

- test — the installed briefing names the `world` verb and its subcommands
- test — the briefing states the no-model-nouns rule
- test — the briefing places the capture step before requirements are derived
- test — `sync-skills` on a project written before the wing reports the briefing updated
- test — `init` on a fresh tree installs the same text
