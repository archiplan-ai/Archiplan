---
affects: [Compiler.Resolver]
outcome: surviving
---

# Preset conns have no def

Edges instantiate conn types the preset defines — there is no `def conn` anywhere in source for
the def sweep to find, however many passes it makes. Arguments arrive bare or lane-tagged and the
engine holds the only lane knowledge.

## Attractor

The two-phase fix overcorrects: a resolver that now *requires* every conn name in the def table
rejects preset conns outright, or infers lanes it cannot know; either way the stdlib vocabulary
stops working and every model that leans on the preset breaks at once.

## Resolution

Holds on v0003 and constrains the fix: the def-less path is legitimate, not a degraded case —
with no lane knowledge a bare argument rides the forward lane, tagged arguments bind as written,
and the engine re-checks arity and patterns downstream. The two-phase resolver keeps this branch
exactly as is; completing the def table changes *when* source defs become visible, never
*whether* one is demanded. The fix's regression suite pins it at the resolver layer: an edge on a
def-less conn resolves identically before and after the split.
