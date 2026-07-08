# `version save` on an unchanged model exits 1

**Kind:** friction (CLI contract) · found by the self-hosting bootstrap
**Status:** resolved 2026-07-08 — a no-op save is a success, exit 0 (`unchanged-saves-close-rounds`)

Refusing to mint on an unchanged model is the design ("versions mint only on semantic change"),
but the refusal shares exit code 1 with genuine failures: `run_version`
(`crates/archi/src/main.rs:398`) maps `Saved::Unchanged` to stderr
`nothing to save: the model is unchanged since v0001` and exit 1 — the same code a compile
failure or a corrupt archive produces.

## Impact

Scripts and CI cannot distinguish "already saved, nothing to do" (benign, often the desired
steady state) from "the save broke". Any pipeline that runs `version save` idempotently has to
parse stderr text to tell them apart, which the stable-code error philosophy elsewhere in the
tool explicitly avoids.

## Fix shape

Either exit 0 with the message (a no-op is a success under at-least-once semantics, matching the
engine's own `noop` outcome for restatements), or reserve a distinct exit code for "unchanged"
and document it in the exit-code table of `requirements/cli.md`.

## Resolution

Took the first option, fused with `rounds-without-model-change-cannot-close` under requirement
`unchanged-saves-close-rounds` (round `lifecycle-pressure`, plan `close-without-minting` @
v0004): an unchanged save exits 0 — closing the open round against the current version when one
exists, reporting `nothing to save: the model is unchanged since <id> and no session is open`
when none does. Messages moved from stderr to stdout because they now report success. Genuine
failures keep exit 1, fenced by the `real-failures-stay-loud` survivor and the two-open-sessions
regression in `crates/archi/tests/version_e2e.rs`. The exit-code table in `requirements/cli.md`
now names benign no-ops successes.
