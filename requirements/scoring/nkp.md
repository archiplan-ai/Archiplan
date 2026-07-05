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

## Slicing

NKP never runs on the raw graph; the landscape is a slice:

- **Preset scaffolding is out.** Everything the model's
  [ontology preset](../modeling-lang/ontology.md) defines is substrate,
  never a landscape node.
- **Exclusion patterns** disqualify nodes. A pattern is edge-shaped over a
  named relation, `<source> <rel> <target>`, with exactly one `_` slot: a
  node is excluded when it can fill the `_` such that the relation holds
  (following the relation's transitive closure when it is `trans`).
  Default, matching the default preset:
  - `_ type_of *` — anything on the left of `type_of`: the epistemic layer.
  - `Data type_of _` — transitive instances of `Data`: data is not software
    structure, even though layer probing calls it a term.

  A pattern naming an unknown rel/node matches nothing and warns
  (`UNKNOWN_EXCLUDE_REF`) instead of failing.
- **Carriers survive.** A connection carrying a data node still couples its
  endpoints; only data as an *endpoint* drops an edge.
- **Scope** — one of:
  - **recursive** (default): all scopes top to bottom; delegation
    applications *fold* — a node realizing its parent's port merges into
    the parent and its couplings re-attach there.
  - **top** — top-level nodes only.
  - **scope `<path>`** — the direct children of one node; no folding, a
    delegated child is a component of that area.
- **Edge types** — optional `--only <type>` list narrows which rel/conn
  types count as coupling.

## Commands / knobs

- **Full NKP** — one JSON blob with the entire landscape analysis.
- **Regime** — just the one-word classification.
- **Hotspots** — the hotspot list as JSON.
- **Corridors** — neutral-corridor detection, parameterized by:
  - `--tau-p` neutrality threshold.
  - `--tau-b` boundary-exposure threshold.
- **Neutrality strategy** (full NKP): `degree_derived` (default) or
  `uniform_p` (`--global-p`).
- **Fitness strategy** (full NKP): `stability_proxy` (default),
  `uniform_random`, or `weight_labeled` — lands with the adaptive-walk
  simulation, which v1 skips (reported in the blob's `notes`, together
  with spectral clustering).

Full conceptual background: `kb/nkp.md`.
