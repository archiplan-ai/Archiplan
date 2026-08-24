---
kind: functional
origin: intent
satisfied-by: [AgentBrief, Scaffold]
deferred:
---

# One door migrates the standing project

`archi-migrate` is the one page for a project that predates a mechanism. It opens with the
measurements that name the gap — `world ls` empty where the model stands, `link ls` counting
rows under `inferred` — and carries the passes inline: the world interview, the journal
triage, and the scrap sweep — a `waves/` folder under a completed plan is dead weight
left by a binary from before the close cleaned up after itself
(`the-plan-cleans-up-after-itself`), measured by `ls -d archi/plans/*/waves` against
`archi plan list`, and removed once; a plan still in flight keeps its folder. What the measurement shows is the pass to run; a tree holding `.fractal/` belongs to
the old client, and one pointer line sends it to `archi-migrate-fractal`, which stays its
own page. The reader does not need to know the name of their staleness to cure it.
`sync-skills` reports an installed skill this binary no longer embeds, so a merge that
retires a page names the orphan instead of leaving it to stand as if current.

## System Context

Three migrations wore three names, and the reader had to diagnose themselves before they
could open the right one. The two in-project passes share one shape — a standing project
predates a mechanism, the gap is measured, closed once, and reported — so they are two
sections of one page behind one triage head. The machine-level move off the old client is a
different kind: a binary swap and an import, with an unmistakable trigger, so it keeps its
page and the head points at it.

The orphan report exists because `sync-skills` creates and updates but never removes: after
this merge every already-deployed project carries `archi-migrate-world/` and
`archi-migrate-links/` folders no binary knows. Silent, they read as current doctrine.

## Satisfy

`AgentBrief` (the `archi-migrate` page: the triage head, the two passes, the fractal
pointer). `Scaffold` (embeds nine skills where there were ten, installs them byte-equal;
`sync-skills` names installed `archi-*` skills absent from the embedded set).

- test — a fresh init installs `archi-migrate` byte-equal, and installs neither
  `archi-migrate-world` nor `archi-migrate-links`
- test — the embedded `archi-migrate` opens with the measurements, carries the world,
  journal and scrap passes, and names `archi-migrate-fractal` as the old client's own page
- test — the scrap pass says a completed plan's `waves/` goes and a live plan's stays
- test — `sync-skills` on a tree holding an installed skill the binary does not embed
  reports it as orphaned, by name, and removes nothing
- test — the world-pass content the standing tests read survives at the new path
