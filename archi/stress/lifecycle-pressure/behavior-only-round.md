---
affects: [Cli, Archive]
outcome: breaking
---

# Behavior-only round

A round's breaking stressor is answered entirely below model resolution — a report filter, an
exit contract, a spec-prose correction: code and docs, no new nodes or edges. The model still
hashes to the version the session pressed. The operator runs `archi version save` to close the
round.

## Attractor

Hit for real by the `signal-pressure` round and filed as
`issues/rounds-without-model-change-cannot-close.md`: sessions close only as a side effect of a
minting save (`close_open_session` runs on `Saved::Written` alone), and the archive declines an
unchanged model — so the save exits `nothing to save` (code 1, indistinguishable from a broken
save: `issues/no-op-save-exits-nonzero.md`), the session dangles open, the one-open-session
discipline jams the next round, and the incidence report that should fire at the close never
fires. Both workarounds are bad: hand-stamp `closed:` and fire incidence manually — the tool
bypassed at its own ceremony — or invent a token model edit to buy a mint, and the archive lies.
Behavior-only rounds are the steady state of a stabilizing model, so the jam recurs by design.

## Resolution

Broke, as filed. Answered this round by letting the no-op save finish the ceremony: on
`Unchanged` the verb closes the open session against the current version — a round may harden
the same version twice — fires the round's incidence report, and exits 0; with nothing open it
reports the no-op and exits 0. This session is the proof: it closes itself through exactly that
path. Derived: unchanged-saves-close-rounds.
