# Model Query API

Structural queries let users request particular slices of a model.

## Subgraph Queries

Filter nodes and edges by:
 - types (nodes)
 - kinds (edges)
 - views (edges and related nodes)
 - carriers (edges carrying the named data — the carrier or rev-carrier is
   the named node, or is classified by it via `type_of` — and related
   nodes: the filter answers "the flow of this data" without a pre-tagged
   view)
 - edge types (edges by rel/conn type name; applications are untyped and
   never pass)
 - scopes to include (e.g a certain flow can be traced only on the higher level or on all inner scopes as well)

 Filters can be composed.

### Result

Result is structured as nodes and edges in JSON. Should preserve meta-information about types, kinds, ports and scopes nesting. At the same time the format should be common enough 
