---
kind: functional
origin: intent
satisfied-by: [Tradeoffs]
deferred:
---

# Priorities weight the read

A trade-off configuration — set explicitly or derived by an auto mode that polls the operator —
declares what the design should favor and what it may spend, and the scoring read consults it so
the landscape verdict is situated in the project's own priorities rather than uniform. An operator
who has said "simplicity over scalability here" sees a read that weighs coupling and neutrality by
that choice; absent any configuration the read is exactly today's.

## System Context

Scoring reads the landscape the same way for every project today (`Nkp`, `Incidence`). The
priorities the operator holds — the first thing a working architect actually decides — have nowhere
to live. This fuses naturally with the scoring landscape's fitness strategies, themselves partly
deferred, and the trade-off configuration is the input those strategies were missing.

## Satisfy

`Tradeoffs` (holds the operator's priorities, set or auto-derived, and hands the scoring read its
weighting; with no configuration present the read is unweighted, byte-identical to today's).

- test — tradeoffs: an explicit configuration weights the landscape read; the same model with no configuration reads exactly as today
- test — tradeoffs: the auto mode derives a configuration from operator answers and the derived weighting is the one applied
- test — tradeoffs: an absent or empty configuration is a valid state — the read never fails for lack of one
