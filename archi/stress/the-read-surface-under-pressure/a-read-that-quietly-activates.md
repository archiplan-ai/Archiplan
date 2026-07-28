---
affects: [Cli, Planner]
outcome: surviving
---

# A read that quietly activates

The named show is implemented lazily: it activates the plan first (writing
`.current`), renders, and calls itself a read.

## Attractor

The read/mutation boundary erodes verb by verb; the guard starts lying, and an
unbound checkout mutates machine state by looking at it.

## Resolution

Held by contract, pinned by test: the named path reaches the loader directly —
`.current` is never written, the router keeps `show` outside the guarded set,
and the verification asserts no `.current` appears after a named show on an
unbound checkout. The nameless form still reads the pointer it never writes.
