# A round that changes no model cannot close its session

**Kind:** gap (round lifecycle) · found dogfooding `incidence-under-stressed-noise`
**Status:** resolved 2026-07-08 — the Unchanged arm finishes the ceremony (`unchanged-saves-close-rounds`)

Sessions close only as a side effect of a *minting* save (`docs::close_open_session` runs on
`Saved::Written`), and saves refuse on an unchanged model. A round whose answers are code and
spec-doc only — the `signal-pressure` round: a report-ergonomics fix with requirement
`under-stressed-names-behavior`, no new nodes or edges — leaves the model at its pinned version,
so `archi version save` exits `nothing to save` and the session dangles open. Session discipline
allows one open session, so the jam blocks the *next* round too. The incidence report that
should auto-fire at the close never fires.

## Observed

    $ archi version save -m "close signal-pressure: ..."
    archi: nothing to save: the model is unchanged since v0004    (exit 1)

Workaround used: hand-stamp `closed: v0004` in the session frontmatter (a text edit into the
tree, philosophy-compliant) and fire `archi incidence --session signal-pressure` by hand.

## Impact

Every behavior-only, docs-only or test-only round hits this — and those are common the moment
the model stabilizes. The alternative is worse: inventing a token model edit per round to buy a
save, polluting the archive with noise versions.

## Fix shape

`Saved::Unchanged` should close the open session against the *current* version (`closed:
<latest>`), fire the incidence report, print what happened, and exit 0 — a round legitimately
hardens the same version twice. This fuses with `no-op-save-exits-nonzero` (its "no-op is
success" option): one change to `run_version`'s `Unchanged` arm covers both.

## Resolution

Run as the loop: `archi/stress/lifecycle-pressure/` pressed v0004 — the jam replay (breaking)
plus three survivors fencing the fix (`no-mint-on-unchanged`: the close borrows the current id,
never mints; `changed-rounds-still-mint`: the `Written` path untouched;
`real-failures-stay-loud`: compile errors and two open sessions keep exit 1). Derived
`unchanged-saves-close-rounds`; implemented via plan `close-without-minting` @ v0004 exactly as
the fix shape asked: `run_version`'s `Unchanged` arm now runs `close_open_session(&root,
&latest)`, fires the incidence report, and exits 0; with nothing open it reports the no-op and
exits 0 (`crates/archi/src/main.rs#run_version`, four regressions in
`crates/archi/tests/version_e2e.rs`, spec in `requirements/versioning.md` and `skills/archi.md`).

The round proved itself: behavior-only by design, it was closed by the binary it built —
`closed: v0004` stamped by the fixed `version save`, incidence fired, exit 0 — the first round
closed without a hand-stamp since the workaround was invented. Links l0100 asserted at the fix
site; the regressions ride as evidence.
