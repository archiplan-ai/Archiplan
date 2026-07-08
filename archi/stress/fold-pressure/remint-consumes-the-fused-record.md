---
affects: [StressDoc, Archive]
outcome: breaking
---

# Remint consumes the fused record

Two writers run complete same-slug rounds — open, press, answer, save — and merge. The
archive collides and prints its recipe; the integrator follows it faithfully: keep the
first-landed entry, `version remint --session <slug>`.

## Attractor

The repair verb for one store quietly finishes destroying another: the round record the
remint re-stamps is a fusion nobody has looked at.

## Resolution

Broke, by recipe. Both sides sealed their round `closed: v0002`, so the fused session file
carries one clean stamp two rounds share and markers only around the charters; `check` after
the archive resolution is green (markers pass for prose) and `remint --session` validates the
session as closed, mints the loser's delta as v0003 — correctly — and re-stamps the shared
seal to v0003. Whichever round the stamp was telling the truth about, it lies now: the
winner's model answers stand in the archive as v0002 while the record of the round that
pressed them out claims v0003, carries the resolver's surviving charter, and holds both
writers' stressors. One sealed round's why is consumed — charter in git archaeology, stamp
overwritten, stressors reattributed — and every verb along the way reported success. The
sequence the recipe implies (archive, then remint) is missing its middle: the record must be
folded, deliberately, before any verb re-stamps it.
