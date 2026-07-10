---
kind: functional
origin: intent
satisfied-by: [Nkp]
deferred:
---

# Criticality is the target

The landscape read places the design on the order–chaos spectrum: the regime — ORDERED,
changes stay local; CRITICAL, the evolvable edge where changes propagate without
cascading, mean coupling roughly one to three; CHAOTIC, every change ripples — alongside
K̄, the mean couplings per component, σ its spread (high against K̄ means a few nodes
hoard the coupling), and P̄, the mean neutrality: the share of the design free to move
with no global ripple. Hotspots are the nodes whose coupling is unusually high — the
highest-risk refactoring targets, candidates for decomposition — and neutral corridors
are regions of low boundary exposure, the safe refactor paths, thresholded by `--tau-p`
and `--tau-b`. Neutrality derives from degree by default or is set uniform
(`--neutrality`, `--global-p`); facets (`--regime`, `--hotspots`, `--corridors`) cut the
JSON report to one answer. The stages v1 skips — the adaptive-walk simulation and
spectral clustering of the dependency matrix — are honest IOUs named in the report's
notes, never silently absent.

## System Context

The regime bands come from Kauffman NKP fitness landscapes; the point is a principled
reading of evolvability, not a score to game. The unimplemented stages stay visible
in-band (`issues/scoring-specs-unimplemented.md` records the gap), so an operator can
tell shipped analysis from aspiration without reading source.

## Satisfy

`Nkp` (K/P metrics, regime bands, hotspot and corridor detection, the notes naming the
skipped stages).

- test — nkp::hub_hotspot_and_corridors
- test — nkp::degenerate_and_disconnected_graphs
