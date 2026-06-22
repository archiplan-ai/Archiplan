# Incidence — Stressor × Component

Archiplan tracks the **incidence matrix**: for every
stressor `s` and every epistatic node `c`, is `c` part of `s`'s
pressure surface? Surfaces couplings the declared edges alone don't show.

## Findings

The incidence analysis produces a set of typed findings:

- **Hyperliminal coupling** — nodes that always react together to the
  same stressors but have no declared edge between them. A hidden
  dependency made visible by the matrix.
- **Stress hotspots** — columns hit by many stressors. A component
  under disproportionate pressure.
- **Compound vulnerabilities** — pairs of *breaking* stressors whose
  union of affected nodes covers an invariant from the initial problem
  statement. Two individually-survivable stressors that together would
  break an initial promise.
- **Under-stressed components** — zero columns: components no stressor
  has touched. Either genuinely invulnerable or (more likely) the
  stress session is blind to them.
- **Merge / extract candidates** — columns with near-identical stress
  responses; signals that two nodes might really be one, or that a
  shared concern should be extracted.

## Reports and numbers

- **Human report** — the default output: matrix plus findings, printed
  after a `version save` that closes a stress session.
  - Set `FRACTAL_REPORT_JSON=1` to get the auto-report as JSON instead.
- **JSON report** on demand — `--json` flag.
- **No-matrix** mode — findings only, omitting the raw matrix.
- **K_hyper density** — a single number (three decimals): how dense
  the incidence matrix is overall.
- **Findings only** — list of typed findings, optionally filtered by
  `--kind` and `--min-severity` (`info` / `warn` / `alert`).
- **Raw matrix** — the S×N matrix as JSON.

## Scoping the analysis

- **Session** — analyze one specific stress session.
- **Since a version** — include only stressors introduced since a given
  version.
- **Exclude pending** — drop stressors with no outcome yet.
- **Tunables** — `--tau-j` (Jaccard threshold for coupling / merge),
  `--tau-d` (density threshold for hotspots), `--path-limit` and
  `--depth` for graph traversal when deciding whether two columns are
  "really" connected through declared edges.

## Auto-fire

The incidence report runs automatically on `version save` when that
save closes a stress session. Full spec: `kb/incidence.md`.