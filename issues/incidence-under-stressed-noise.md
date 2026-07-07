# Incidence drowns its signal in under-stressed data nodes

**Kind:** friction (analysis ergonomics) · found by the self-hosting bootstrap

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
