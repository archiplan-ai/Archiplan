# NKP Landscape

Fractal treats the epistatic graph as a Kauffman NKP fitness landscape
and reports where on the order ↔ chaos spectrum the architecture sits.

## Core idea

Evolvable architectures live at **criticality**: enough coupling that
changes propagate meaningfully, but not so much that every change
ripples everywhere. NKP gives a principled reading of where the design
falls.

## What it reports

- **Regime** — ORDERED, CRITICAL, or CHAOTIC. CRITICAL is the target.
- **Mean connectivity (K̄)** — average coupling per node. Sweet spot
  is roughly 1.0–3.0.
- **Mean neutrality (P̄)** — how much of the design space around each
  node is effectively equivalent.
- **Hotspots** — nodes whose coupling is unusually high; candidates
  for decomposition.
- **Neutral corridors** — regions of the design space where changes
  have low boundary exposure. Safe refactor paths.

## Commands / knobs

- **Full NKP** — one JSON blob with the entire landscape analysis.
- **Regime** — just the one-word classification.
- **Hotspots** — the hotspot list as JSON.
- **Corridors** — neutral-corridor detection, parameterized by:
  - `--tau-p` neutrality threshold.
  - `--tau-b` boundary-exposure threshold.
- **Fitness strategy** (full NKP): `stability_proxy` (default),
  `uniform_random`, or `weight_labeled`.
- **Neutrality strategy** (full NKP): `degree_derived` (default) or
  `uniform_p`.
- **Layer** and **only-edge-types** — the same filters as `check`.

Full conceptual background: `kb/nkp.md`.
