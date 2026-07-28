---
kind: functional
origin: intent
satisfied-by: [Planner, PlanState]
deferred:
---

# A plan is a folder of records

A plan lives as a folder of markdown records under `archi/plans/<name>/`: the
charter `<name>.md` (the problem and the stack), one file per task, and
`scenarios.md` — read and merged like every other doc. Lifecycle lives apart in
`state.json` — state, closed waves, the pinned version and its content hash —
written by verbs alone. A legacy `plan.json` stays readable forever, read-only;
a plan carrying both forms at once is an error naming which to keep.

## System Context

The plan was the tree's last JSON blob: unreadable in review, merged as a lump,
one parse error swallowing the whole file. Folder-of-records is the shape the
rest of the spec already has — intents, rounds — so plans inherit the same
parser family, located diagnostics, text merges and the verdict gate. Splitting
lifecycle into `state.json` keeps content files still when waves close (diffs
stay meaningful) and keeps the never-hand-edit surface one small file. Dual
form matters mid-flight: the plan driving this very migration is a legacy
`plan.json` and its lifecycle verbs keep working.

## Satisfy

`Planner` (loads the folder into the same in-memory plan every read verb
already serves; renders records back only through mint verbs); `PlanState`
(the lifecycle file, verbs-only).

- test — a folder plan round-trips: mint, fill, verify, start, close read the same plan the files spell
- test — a legacy plan.json loads read-only and its lifecycle verbs still advance it
- test — a plan carrying both plan.json and the folder refuses loudly, naming the choice
