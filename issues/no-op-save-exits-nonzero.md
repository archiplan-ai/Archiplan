# `version save` on an unchanged model exits 1

**Kind:** friction (CLI contract) · found by the self-hosting bootstrap

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
