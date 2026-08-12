---
kind: functional
origin: stressor(the-removal-is-a-wall-not-a-walk)
satisfied-by: [DocMint]
deferred:
---

# The refusal is an ordered continuation

The removal's refusal lists the commands that clear it, in the order they must run: the
plan verbs first, then the link retirement, then the dependant facts. Each line is a
runnable command with its arguments filled in. The refusal stays a refusal — nothing
cascades, and no verb retires a link or closes a plan on the operator's behalf.

## System Context

Deletion is the operation the world exists for, and a refusal that lists three blockers
without saying how to clear them makes deletion the most expensive thing an operator can
attempt. The rational response to an expensive deletion is to keep the false fact and work
around it, which turns the world into the accumulating pile it was built to give deletion
pressure to. Naming the continuation costs a few lines of rendering and it is already the
contract every refusal in this tool holds. `removal-names-the-code-it-strands` settled that
nothing cascades — a link is a recorded human assertion — and that stands: the operator
runs the commands, the tool writes them out.

## Satisfy

`DocMint` (renders the three pre-flights as an ordered command list with arguments,
derived from the blockers it already resolved).

- test — a fact blocked by a plan, a link and a dependant prints three ordered commands
- test — each printed command runs as written and clears its blocker
- test — a single blocker prints a single command
- test — the order puts plan verbs before link retirement before dependants
- test — no blocker is cleared by the refusing verb itself
