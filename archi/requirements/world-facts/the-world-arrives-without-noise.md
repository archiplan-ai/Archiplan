---
kind: functional
origin: stressor(the-standing-project-has-no-world)
satisfied-by: [Scaffold, DocMint]
deferred:
---

# The world arrives without noise

A project that upgrades into a binary that knows the world keeps a green `check`. No
finding fires for a node that no fact covers, and no verb requires `archi/world/facts/` to
exist. The first `world add` creates the folder. The briefing that `init` and
`sync-skills` install is where the world is announced, so an agent learns of it by
reading its instructions and not by meeting an error.

## System Context

Every tree in the world is an upgrading tree, and the world is optional by design: a
node justified from the architecture side alone is a normal node, not a defect. A
finding per uncovered node would fire hundreds of times on the first run after an
upgrade, and the operator would mute the class that day and never hear it again. The
briefing is how this tool has always taught its agents what exists, and it costs
nothing at the tree.

## Satisfy

`Scaffold` (the briefing names the world, its verb and the shape of a fact; `init` stays
create-only and adds nothing to a standing tree). `DocMint` (the first `world add`
creates `archi/world/facts/` on the way to writing the file).

- test — a tree with no `archi/world/facts/` passes `check` with no world finding
- test — `world add` on a tree with no `archi/world/facts/` creates it and writes the file
- test — no finding fires for a model element that no fact covers
- test — the installed briefing names the `world` verb and the fact's headings
- test — `sync-skills` on a standing project reports the briefing as updated
