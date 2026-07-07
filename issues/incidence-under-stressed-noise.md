# Incidence drowns its signal in under-stressed data nodes

**Kind:** friction (analysis ergonomics) · found by the self-hosting bootstrap
**Status:** resolved 2026-07-07 — behavioral default + `--all-terms` (`under-stressed-names-behavior`)

The incidence report emits one `under_stressed` info finding per never-pressed term. After the
first stress round over the self-model, that was 31 of 39 columns — the majority pure data nodes
(`Tokens`, `Ast`, `Report`, `SubgraphResult`, …) that no stressor would ever sensibly press.

## Observed

The auto-fired report on the save that closed `first-pressure`: two genuinely interesting
alerts (compound vulnerabilities over the `capture-at-the-join` surface) and one warn
(`Links` hotspot) sit above a 31-line wall of `[info] under-stressed` entries.

## Impact

The actionable tail — which *components* went unpressed — is buried under vocabulary nodes. NKP
already solved this exact problem for its slice: its default class filter drops data-classified
terms and preset members (`nkp::default_slice_keeps_behavior_drops_data_types_and_preset`);
incidence's under-stressed sweep has no equivalent.

## Fix shape

Give the under-stressed sweep a default class filter mirroring NKP's (drop terms classified
`Data`, keep everything behavioral), with a flag to widen back to all terms. `--json` output can
stay complete either way.

## Resolution

Run as the loop: `archi/stress/signal-pressure/` pressed v0004 — the wall replay (breaking) plus
two survivors that fence the fix (`pressed-data-still-counts`: only the *emission* consults the
filter, a pressed data column counts everywhere; `unclassified-terms-stay-loud`: the boundary is
exactly `Data`'s `type_of` closure, so no ontology means no muting). Derived
`under-stressed-names-behavior`; implemented via plan `filter-under-stressed` @ v0004:
`IncidenceConfig.all_terms` + a muted set consulted only in the zero-column branch
(`crates/modeling-lang/src/incidence.rs#analyze`), `--all-terms` on the CLI, spec updated in
`requirements/scoring/incidence.md` and `requirements/cli.md`. On this repo the sweep dropped
from 47 findings to 25 behavioral ones, zero pure-data terms. Links l0076/l0077/l0083 asserted;
the regression rides as evidence (l0091).

The round itself was behavior-only, which exposed and filed
`issues/rounds-without-model-change-cannot-close.md` (session hand-stamped `closed: v0004`, the
verb-based close fuses with `no-op-save-exits-nonzero`).
