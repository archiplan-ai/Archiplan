---
kind: functional
origin: stressor(the-agent-arrives-blind)
satisfied-by: [Scaffold]
deferred:
---

# The agent arrives briefed

Init installs the operating knowledge beside the tree it scaffolds: the workflow
and merge skills land verbatim — byte-equal to the binary's embedded copies —
under `.claude/skills/archi/SKILL.md` and `.claude/skills/archi-merge/SKILL.md`,
and `CLAUDE.md` carries a fenced archi block naming the source dir, the `archi
check` loop, `archi search` and the installed skills. A `CLAUDE.md` that already
exists gains the block by append — its own prose does not move; a file already
carrying the fence is left as found.

## System Context

The schemas and lifecycle rules live in archiplan's repository; the projects it
models start empty of them. Skills are how an agent loads a workflow on demand,
and `CLAUDE.md` is what a session reads unprompted — between them the briefing
survives the clone, and the fence marks the one region init may claim again on a
later day (`the-installed-skill-drifts` presses the ownership question; create-only
answers it).

## Satisfy

`Scaffold` (embeds both skills at build time, installs them create-only, and
appends or verifies the fenced CLAUDE.md block).

- test — a fresh init leaves both SKILL.md files byte-equal to the embedded copies
- test — an existing CLAUDE.md keeps its prose and gains the fence exactly once across two runs
