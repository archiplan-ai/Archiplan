---
kind: functional
origin: intent
satisfied-by: [Planner, Sessions, Archive]
deferred:
---

# Pins survive a remint

A pinned version is verified by content, not id alone. When a remint changes what an id
resolves to, every stale pin — in plans and stress rounds — surfaces as a finding naming the
repair verb, never as silent reinterpretation of foreign content.

## System Context

Remint keeps the first-landed id and re-mints the loser as the next one
(remint-rejoins-the-lineage) — so a plan pinned on the losing branch would silently point at
someone else's render. The Archive already hashes every entry; the pin must carry that hash so
check can compare. Repair stays a verb: plan repin.

## Satisfy

`Planner` stamps the pinned version's content hash into the plan and `Sessions`' closing
stamp carries one; `Archive` serves the entry hash `check` compares against — a mismatch is
a finding naming `plan repin` or `version remint --session`, never silent reinterpretation.

- test — a doctored plan pin surfaces as a stale-pin finding naming the repair (`a_doctored_plan_pin_surfaces_as_a_stale_pin_finding`)
- test — a doctored session stamp surfaces the same way (`a_doctored_session_stamp_surfaces_as_a_stale_stamp_finding`)
