---
kind: functional
origin: intent
satisfied-by: []
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
