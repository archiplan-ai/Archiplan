# A round that changes no model cannot close its session

**Kind:** gap (round lifecycle) · found dogfooding `incidence-under-stressed-noise`

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
