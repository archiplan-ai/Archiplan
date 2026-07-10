---
kind: functional
origin: intent
satisfied-by: [Query]
deferred:
---

# Queries compose filters

One read slices the model by any composition of filters, each optional and unrestricting
when absent: `types` keeps the transitive instances of the listed types; `kinds` keeps
edges of the listed kinds; `views` keeps the listed views' edges plus the nodes they
relate; `scopes` opens the named scopes, the empty list meaning the top level alone;
`carriers` follows a datum — the edges whose carried node is the named node or classified
by it, answering "the flow of this data" without a pre-tagged view; `edge-types` slices
by rel/conn type name, applications never passing (they are untyped). The result is the
slice as plain nodes and edges with its meta intact — types, kinds, ports, view tags,
nesting encoded in paths — every statement replayable as source. Filter names are
validated: an unknown type, view or scope is a rejection, not an empty answer.

## System Context

Agents ground themselves by querying before editing — `agents-read-lowered-statements`
is the envelope these reads ride — and a filter that silently matched nothing would teach
them the model is smaller than it is. `check` rides the same read door, reporting
model-completeness findings.

## Satisfy

`Query` (composes the six filters over the compiled graph, validates filter names,
preserves meta; its audit door serves `check`'s findings).

- test — semantics::query_scopes_open_chains_and_subtrees
- test — semantics::query_kinds_filter_edges_and_compose_with_scopes
- test — flow_filters::carriers_slice_the_flow_of_a_datum
- test — flow_filters::edge_types_slice_by_name_and_never_pass_applications
- test — semantics::query_filters_are_validated
- test — worked_example::query_returns_the_slice_with_meta
