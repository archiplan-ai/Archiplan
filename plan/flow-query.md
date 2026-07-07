# Flow-query primitives — implementation plan

What can't be asked today: `archi query`/`read` retrieve slices by
`types`/`kinds`/`views`/`scopes`, so a **named** flow (a view) comes back
whole — but "every edge carrying `CredHash`" (no carrier filter), "only
`wire` edges" (no edge-type-name filter), and "everything downstream of
`Orders.events`" (no traversal) are unanswerable server-side; the client
must fetch a broad slice and walk it. Three additive primitives close the
gap: two query filters and one `trace` read statement. Everything is
read-only and rides the existing envelope — vocabulary, error codes and
result kinds are append-only, as the contract requires.

## Design decisions (settle before coding — they shape the schema)

1. **Carrier matching includes classification.** A `carriers` filter names
   a node; a connection edge passes when its `carrier` or `rev_carrier`
   *is* that node or is classified by it through the `type_of` closure
   (`c == t || rel_holds(type_of, t, c)`) — the same
   name-a-type-to-mean-its-instances convention the `types` node filter
   already uses. Edges with no carrier (relations, applications,
   carrier-less conns) never pass a carriers filter: it is a restriction.
2. **A carriers filter admits related nodes, like views.** The `related`
   set in `query::subgraph` (today computed only for a views filter:
   attachments + carriers of passing edges) generalizes to "views or
   carriers filter present". Consequence: `query {"carriers":
   ["CredHash"]}` returns the *flow subgraph of that data* — the user-facing
   point of the feature.
3. **Trace is node-granular.** Arriving at a node continues on any edge
   leaving it — a node is a black box, so flow-through-node is the honest
   over-approximation for an architecture model. `from` may name a port
   (`Orders.events`) to seed the first hop precisely. Port-strict traversal
   is a non-goal (revisit only with evidence).
4. **Trace follows flow edges by default.** Default traversable kinds are
   `connection` + `application` (apps are how a flow crosses a scope
   boundary: outer port ↔ inner port, both directions). Relations are
   classification, not flow — traversable only by explicit
   `"kinds": ["relation", …]`. Directed conns traverse source→target
   downstream (reversed upstream); undirected conns traverse both ways in
   either direction.

## Phase 1 — filter extensions (modeling-lang) — landed

- `Statement::Query` gains `carriers: Option<Vec<String>>` and
  `edge_types: Option<Vec<String>>`; `allowed_keys("query")` gains both
  (strict keys stay strict). Absent = no restriction; the empty list is the
  most restrictive filter, uniform with `scopes`.
- `query::SubgraphFilter` gains `carriers: Option<Vec<NodeId>>`,
  `edge_types: Option<BTreeSet<…>>`. `edge_pass` extends: type-name match
  on the edge's rel/conn name (applications are untyped — an `edge_types`
  filter excludes them); carrier match per decision 1. `related` per
  decision 2.
- Engine resolution in `apply`: carriers through `resolve_abs` (unknown →
  `E_UNKNOWN_NAME`, ref kind `node`); edge type names against the defined
  rel/conn tables (unknown → `E_UNKNOWN_NAME`, ref kind `edge-type` —
  new ref kind, append-only).
- Tests (modeling-lang, house style): carrier exact vs via-type matching;
  rev_carrier matches; relations/apps excluded by carriers; edge_types
  slices by name and drops apps; empty-list = matches nothing; unknown
  names error with refs; composition with views/scopes/kinds is AND.

## Phase 2 — CLI for the filters — landed

- `archi query` grows repeatable `--carrier <path>` and
  `--edge-type <name>`; both fold into the composed statement only when
  present. `archi read` needs no work — the statements parse already.
- e2e (`read_e2e.rs`): `query --carrier` returns the carrying edges plus
  their endpoint and carrier nodes; `--edge-type wire`; unknown names exit
  1 with the human one-liner.
- Spec: `modeling-lang/queries.md` filter list + the carrier-admits-nodes
  sentence; `cli.md` query synopsis.

## Phase 3 — the `trace` read statement (modeling-lang)

```json
{ "stmt": "trace",
  "from": "Orders.events",          // node, or node.port to seed the first hop
  "to": "Billing",                  // optional sink: enumerate paths from→to
  "direction": "downstream",        // downstream (default) | upstream
  "kinds": ["connection"],          // optional; default connection+application
  "edge_types": [], "carriers": [], "views": [], "scopes": [],  // restrict traversable edges, as in query
  "max_hops": 8,                    // optional; default unbounded (visited-set terminates)
  "path_limit": 64                  // optional; enumeration bound when `to` is set
}
```

- **Result** — new `Outcome::Trace`, Graph-shaped plus paths:

```json
{ "result": "trace",
  "nodes": [ … ],                   // the reachable cone, GraphNode shape
  "edges": [ … ],                   // traversed edges, GraphEdge shape
  "paths": [ [0, 2, 5] ],           // only when `to` is set: edge indexes into `edges`, in hop order
  "truncated": false }              // some path or hop was cut by a bound
```

- **Semantics**: BFS from `from` over passing edges (all query filters
  restrict the traversable set — trace composes with Phase 1); visited-set
  cycle termination; `to` set → DFS path enumeration bounded by
  `max_hops`/`path_limit`, `truncated` flagged on any cut. No path is an
  answer, not an error: `"paths": []`. Unknown `from`/`to`/filter names →
  `E_UNKNOWN_NAME` with refs. `scopes` controls which scopes traversal may
  descend into, exactly as the query filter opens scopes.
- **Envelope**: the engine's read whitelist becomes
  `Query | Check | Trace`; writes stay protocol errors. Statement parsing:
  `allowed_keys("trace")`, `direction` validated (`E_PARSE` otherwise).
- Tests: linear chain down/up; fan-out cone; cycle terminates; descent
  through an application into an inner scope and back out; undirected edges
  traverse both directions; `to` enumerates multiple paths in hop order and
  respects `path_limit`+`truncated`; `max_hops` cuts the cone; filters
  restrict traversal (a carrier filter yields the data's path only);
  port-seeded `from` takes only that port's edges on hop one; envelope
  accepts trace in a batch and rejects it nowhere.

## Phase 4 — CLI `archi trace`

- `archi trace <from> [--to <path>] [--direction downstream|upstream]
  [--depth <n>] [--path-limit <n>] [--kind <k>]... [--edge-type <t>]...
  [--carrier <p>]... [--view <v>]... [--scope <p>]... [--at <id>]` —
  positional `from` (parse_args admits positionals for the verb), `--to`
  and `--depth`/`--path-limit` reuse the existing flags (`--depth` maps to
  `max_hops`), output is the single `trace` outcome unwrapped, errors as
  one-liners, exit 1 — mirroring `query`.
- e2e: trace through the model via the binary, `--at` against a sealed
  version, no-path exits 0 with empty paths.
- Spec: `queries.md` gains a Trace section (statement, result, semantics);
  `agent-interface.md` read-statement list and result objects gain trace;
  `cli.md` command entry.

## Non-goals

- Port-strict traversal; `direction: both` (run two statements in one
  batch); persistent/named traces — a recurring flow worth keeping is a
  **view**, tagged in source: trace answers ad-hoc questions, views record
  architecture; any mutation surface — trace/read stay reads by
  construction.

## Order and size

Phases land independently green, 1 → 2 → 3 → 4; 1–2 are small (filter
plumbing + tests), 3 carries the substance (traversal + result type +
statement validation), 4 is plumbing. Each phase: module tests in the house
style; 2 and 4 extend `crates/archi/tests/read_e2e.rs`. Acceptance for the
feature as a whole: "retrieve the flow of `CredHash`" is one statement
(`query` + carriers), "what is downstream of `Orders.events` at v0003" is
one statement (`trace` + `--at`), and both ride `archi read` batches
unchanged.
