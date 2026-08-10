---
kind: functional
origin: stressor(the-findings-crowd-the-worklist)
satisfied-by: [DocsCompiler]
deferred:
---

# One fact reports one finding

A fact reports at most one line. `world_state` names the fact once and lists every state
it holds — ungrounded, uncovered, speaks-the-model, orphan — in one place. The graph-shaped
reports stay separate, because they are about the wing and not about a fact: the `uses`
cycle error and `world_chain_deep`.

## System Context

Findings are the worklist in this tool, and a worklist is read while it is short. One
honest early fact — observed in a conversation with no locator, written before the model
reached it — held three of the six kinds the wing grew over five rounds, on a tree where
nothing was wrong. An operator meeting three lines per healthy file learns that world
findings are noise, and `world_ungrounded` is the one signal standing between the wing and
a fluent invention. Collapsing to a line per fact keeps every state and costs the reader
one glance instead of three.

## Satisfy

`DocsCompiler` (collects the per-fact states into one `world_state` finding carrying the
fact and its list; graph findings stay their own kinds).

- test — a fact holding three states reports one line naming all three
- test — a fact holding one state reports one line naming it
- test — a healthy fact reports nothing
- test — the `uses` cycle error and `world_chain_deep` are unaffected
- test — the line never changes the exit code
