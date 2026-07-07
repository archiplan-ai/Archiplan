# Incidence — Stressor × Component

Archiplan tracks the **incidence matrix**: for every stressor `s` in
scope and every epistatic term `c` of one **frame** version, is `c` on
`s`'s pressure surface? Rows come from stress sessions
([stressing.md](../stressing.md)), columns from an archived model
version ([versioning.md](../versioning.md)); the matrix surfaces
couplings the declared edges alone don't show.

## Surfaces and the frame

An affects entry names a term or a type: a **term is its own surface; a
type expands to the terms its `type_of` closure transitively
classifies** — the one expansion rule `satisfied-by` entries share
([requirements.md#satisfy](../requirements.md#satisfy)). Expansion runs
against the stressor's *own* session's pinned version, reconstructed
from the archive — closed sessions re-validate on analysis exactly as
[stressing.md#compile](../stressing.md#compile) demands, and an affects
path that does not resolve there is `E_MODEL_REF`.

The expanded term paths join against the **frame**: the newest pinned
version in scope. For a single session the frame *is* its pinned
version. Across sessions, affected terms the frame no longer knows are
dropped **visibly** — per stressor under `scope.dropped`, with the
`DROPPED_AFFECTS` warning: version drift is data, not noise.

## Findings

Typed and severity-tagged (`info` < `warn` < `alert`); kinds are
append-only, and the serialized names below are the `--kind` filter
currency:

| kind | severity | fires when |
|------|----------|------------|
| `compound_vulnerability` | alert | two *surviving* stressors, neither of which alone covers an invariant, whose union of affected terms does — individually answered, jointly a broken initial promise |
| `hyperliminal_coupling` | warn | two columns with near-identical stressor sets (Jaccard ≥ τ_J over ≥ 2 shared stressors) and *no* declared path between them — a hidden dependency made visible by the matrix |
| `stress_hotspot` | warn | a column pressed by ≥ 2 stressors making up a τ_D share of the scope — a component under disproportionate pressure |
| `merge_candidate` | info | the same response similarity as hyperliminal coupling but *over* a declared path — two nodes that might really be one, or a shared concern worth extracting |
| `under_stressed` | info | a zero column: no stressor touches the term — genuinely invulnerable or (more likely) the stress work is blind to it |

The under-stressed sweep names **behavior** by default: zero columns in the `type_of` closure of
`Data` — the boundary [NKP's default slice](nkp.md) already draws — emit no finding, so the
report's tail is the list of unpressed components rather than a wall of vocabulary;
`--all-terms` widens the sweep back to every zero column. The filter lives at the emission site
alone: column construction, the matrix and every other finding always see all terms — a
*pressed* data column counts everywhere regardless — and a model with no `Data` (or one whose
`Data` classifies nothing) mutes nothing.

An **invariant** is the satisfaction claim of an intent-origin
requirement — the `satisfied-by` elements of a promise derived directly
from the initial problem statement
([requirements.md#origin](../requirements.md#origin)) — expanded against
the frame. Stressor-derived claims answer pressure; they do not seed
compound analysis.

A **declared path** walks the undirected structure the model actually
declares — relation edges (never `type_of`: its source is a type),
connections between the port-owning nodes, applications, and
containment — hop-bounded by `--depth` and node-budgeted by
`--path-limit`. An exhausted budget assumes the pair connected,
suppressing a finding rather than fabricating one, and warns
`PATH_LIMIT_HIT`.

## Reports and numbers

- **Human report** — the default `archi incidence` output: the matrix
  plus findings.
- **JSON report** — `--json`: one envelope carrying the scope's
  sessions, the frame, the matrix and the findings.
- **No-matrix** — `--no-matrix`: either format without the raw matrix.
- **K_hyper density** — `--k-hyper`: a single number (three decimals),
  ones / (S×N) — how dense the incidence matrix is overall.
- **Findings only** — `--findings`, a JSON list; `--kind <kind>`… and
  `--min-severity info|warn|alert` filter the findings in every mode.
- **Raw matrix** — `--matrix`: the S×N matrix as JSON.

## Scoping the analysis

- **Default** — the open session; with none open, the latest-closed one.
- **Session** — `--session <slug>`: one specific session, open or closed.
- **Since a version** — `--since <id>`: every session pressing that
  version or a later one, joined against the newest pinned version.
- **Exclude pending** — `--exclude-pending`: drop stressors with no
  outcome yet.
- **Tunables** — `--tau-j` (Jaccard threshold for coupling / merge,
  default 0.8), `--tau-d` (density threshold for hotspots, default 0.5),
  `--depth` (declared-path hops, default 2), `--path-limit` (traversal
  node budget, default 4096).

A stress tree that does not compile refuses to be analyzed — the
diagnostics come back instead
([stressing.md#compile](../stressing.md#compile)).

## Auto-fire

The report runs automatically on the `archi version save` that closes a
stress session, over the finished round
([versioning.md#versioning--stressing](../versioning.md#versioning--stressing)).
Its failure is a warning, never a failed save. Set `ARCHI_REPORT_JSON=1`
to get the auto-report as JSON instead.

## Why this shape

- **Surviving pairs, not breaking ones.** A breaking stressor already
  bent the architecture and derived its requirements
  ([stressing.md](../stressing.md)); the compound finding exists for the
  promises that break only in combination — and only pairs where
  neither side covers alone say anything the single rows didn't.
- **One similarity signal, two readings.** Near-identical columns over
  a declared path corroborate the structure (merge them?); the same
  evidence with no path contradicts it (what couples them?). The split
  is exactly the declared-edge traversal.
- **The frame is an archived version, never the live tree.** Analyses
  over past sessions reproduce bit-for-bit years later; what the live
  tree thinks today is the next session's business.
