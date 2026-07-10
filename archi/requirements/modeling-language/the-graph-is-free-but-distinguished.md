---
kind: functional
origin: intent
satisfied-by: [Engine]
deferred:
---

# The graph is free but distinguished

A model is nodes and edges with no fixed metamodel — yet neither is uniform. A node may
expose ports (named attachment points) and contain a scope (a nested subgraph of inner
nodes). Every edge belongs to exactly one of three kinds that decide where it attaches: a
relation relates two nodes at their boundary; a connection lands on a surface port at
each end; an application maps an outer node's port to a port of a direct child, crossing
the scope boundary and inheriting its direction from the ports it joins. Relation and
connection are open kinds — users declare as many named types of them as they need;
application is a single built-in type, never declared, only used.

## System Context

The graph must express arbitrary systems, so structure the language would otherwise
hard-code — layers, component taxonomies — arrives as user-declared types and the preset
ontology (`one-ontology-everywhere`) instead of grammar. Relations and connections
operate between nodes of one scope; the application is the only construct that reaches
across a boundary, and only one hop down.

## Satisfy

`Engine` (nodes, ports, scopes and the three edge kinds are its vocabulary; a connection
joining nodes of different scopes, or an application whose inner end is not a direct
child, rejects as E_CROSS_SCOPE).

- test — worked_example::worked_example_applies
- test — errors::e_cross_scope_connection
- test — errors::e_no_outer_port
