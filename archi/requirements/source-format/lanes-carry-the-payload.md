---
kind: functional
origin: intent
satisfied-by: [Compiler.Parser, Compiler.Resolver]
deferred:
---

# Lanes carry the payload

A conn definition reads `source LANES target`: direction means initiation, and
carried slots ride on the lanes, so one line states who calls whom and what data
travels each way. On the edge, a lane whose pattern is an exact node may be omitted
— the compiler fills it in, and lowered statements are always fully explicit — while
`*` and classified lanes must be named, `E_CARRIER_REQUIRED` at compile time with
the edge's span.

The lane forms and what they lower to:

| form | meaning | statement fields |
|------|---------|------------------|
| `* -> *` | directed, no payload | — |
| `* ->P *` | directed, forward payload | `carrier` |
| `* ->P, <-Q *` | request/response | `carrier`, `rev_carrier` |
| `* ->, <-Q *` | pull: initiate forward, payload back | `rev_carrier` |
| `* <-> *` | undirected | — |
| `* <->P *` | undirected, single payload | `carrier` |

Carried slots are patterns: a bare path is an exact node, `*` is any, `(X rel *)`
is classified. Illegal: a leading or lone `<-` lane (flip the ends instead), and
`<->` combined with a reverse lane. Port sides are unchanged — the source port is
`source` even when payload flows back.

```
def conn login := * ->LoginForm, <-AuthResponse *   // bidirectional request/response
def conn send  := * ->(Message type_of *) *         // classified payload
```

A connection edge is `Node.port type Node.port` — a conn end's last segment is the
port, the prefix is the node. Carried nodes go in parens after the type name; a
bare argument is legal only when exactly one lane carries, and two arguments must
tag their lanes:

```
UI.login login AuthService.handle_login in login_flow   // carriers inferred (both lanes exact)
A.out send(OrderCreated) B.inbox                        // bare: binds the single carrying lane
A.req rpc(->Query, <-Result) B.serve                    // tagged: one per lane
```

Relations keep the statement layer's syntax minus semicolons — the semantics
(patterns, transitivity, shape checks) stay with the modeling-language cards:

```
def rel trans of_sort := * -> *
def rel has_pii := (Service type_of *) -> (Data type_of *)

Service type_of AuthService
Payments fails_via Orders in fault_prop
```

Views are a name and a tag: `def view login_flow` declares one, and any edge joins
views with a trailing `in` clause — `UI.login login AuthService.handle_login in
login_flow, audit`.

## System Context

Shape checking — do the ends and carried nodes match the type's patterns — belongs
to the engine (`one-semantic-authority`, `E_SHAPE_VIOLATION`); the surface owns
only the notation and the inference. Carrier inference reads conn lane patterns out
of the completed def table, so it works whatever module the def sits in
(`uses-see-every-def`).

## Satisfy

`Compiler.Parser` (parses every lane form, rejects illegal ones, and separates
carrier from target by lookahead) and `Compiler.Resolver` (maps carrier arguments
onto lanes — bare onto the single carrying lane, tagged onto theirs — fills omitted
exact-node lanes, and raises `E_CARRIER_REQUIRED` at the edge's span when a lane is
genuinely uninferable).

- test — parser::all_conn_lane_forms_parse, parser::rel_defs_parse
- test — parser::two_carriers_must_tag_their_lanes
- test — resolve::carrier_arguments_map_onto_lanes
- test — bidir_conns::rev_only_lane_models_pull, bidir_conns::undirected_types_reject_a_rev_lane
- test — bidir_conns::lane_arity_is_checked_per_lane, bidir_conns::rev_carrier_is_part_of_edge_identity
