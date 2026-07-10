# Trade-off configuration exists only as an aspiration

**Kind:** missing feature · recorded at the legacy-requirements migration (2026-07-10)

The legacy spec floated configuring architectural trade-offs up front — what matters most
in the current case, what can be sacrificed (e.g. scalability traded for simplicity and
speed of development) — with an auto mode where the agent polls the user and derives a
suitable configuration itself. Never designed, never pressed through the loop, no code
behind it.

## Impact

None today: no verb or analysis consumes a trade-off configuration. The idea's natural
consumers would be the scoring layer (weighting the landscape read) and stress-round
prioritization.

## Fix shape

Recorded so it expires by being seen, not by being forgotten. If it becomes real work, it
enters as an intent with a problem statement — most likely fused with the scoring
landscape's fitness strategies, which are themselves partly deferred
(`scoring-specs-unimplemented.md`).
