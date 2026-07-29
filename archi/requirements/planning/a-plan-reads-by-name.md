---
kind: functional
origin: intent
satisfied-by: [Cli, Planner]
deferred:
---

# A plan reads by name

`plan show <name>` renders any plan without activating it: the name reaches the
loader directly, `.current` is never written, and the read answers on an unbound
checkout — a reviewer on the receiving branch looks at a landed plan with no
seat and no side effect. Without a name the verb keeps serving the active plan.

## System Context

Activation (`plan use`) is a mutation — it writes the current pointer and joins
the seat's binding — so it is guarded; looking is not. Before this claim the
only path to a specific plan's render went through activation, which an unbound
checkout correctly refuses: landed plans were readable only as raw files. The
record form made the files legible, and the verb completes the pair: files for
the editor, the rendered projection for the terminal.

## Satisfy

`Cli` (the optional name on `plan show`, a free read at the router); `Planner`
(the loader serves any named plan — record or legacy — without touching
`.current`).

- test — `plan show <name>` renders a plan on an unbound checkout and writes no `.current`
- test — `plan show` without a name keeps serving the active plan; an unknown name lists the plans
