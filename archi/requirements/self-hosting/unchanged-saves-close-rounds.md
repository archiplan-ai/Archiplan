---
kind: functional
origin: stressor(behavior-only-round)
satisfied-by: [Cli]
deferred:
---

# Unchanged saves close rounds

`archi version save` on an unchanged model is a success, not a failure: with a session open it
closes the round against the *current* version — a round may harden the same version twice —
fires the round's incidence report, and exits 0; with no session open it reports the no-op and
exits 0. Minting stays reserved for semantic change (`versions-mint-on-meaning`), and only
genuine failures — compile diagnostics, a corrupt archive, two open sessions — exit nonzero.

## System Context

Sessions close as a side effect of `version save`, and behavior-only rounds are the steady state
of a stabilizing model: the close ceremony — the `closed:` stamp and the incidence report — must
not depend on the model having moved. Replayed from
`issues/rounds-without-model-change-cannot-close.md` fused with
`issues/no-op-save-exits-nonzero.md`.

## Satisfy

`Cli` (the `version save` arm composes the archive's `Unchanged` answer with the round ceremony:
`close_open_session` against the latest version id, incidence fired over the just-closed round,
exit 0 for the close and for the bare no-op alike; the `Written` path and every failure path are
untouched).

- test — version_e2e: open session + unchanged model → exit 0, session stamped `closed:` with the current id, incidence report in stdout, archive unchanged
- test — version_e2e: no open session + unchanged model → exit 0, the no-op reported, nothing written
- test — version_e2e: open session + changed model → mints and closes against the minted id, as before
- test — version_e2e: two open sessions + unchanged model → exit 1, the jam stays loud
