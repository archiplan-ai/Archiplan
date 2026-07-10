---
kind: functional
origin: intent
satisfied-by: [Engine, Query]
deferred:
---

# Views tag edges into stories

A view is a named perspective — one slice of edges telling one story about the system:
data flow, fault propagation, a feature's path. Views are declared before use; an edge
joins one or more views at instantiation with `in`, and restating the edge with more
views extends its tags. Tagging is per edge, never per type; a node belongs to a view
only through its incident edges; an untagged edge is visible to unfiltered queries alone.
Applications are untagged plumbing — they belong to the views of the connection edges
they route. Preset edges take no tags at all (E_STDLIB_PROTECTED): tags on substrate
would not survive a dump replay.

## System Context

Views are the query surface's currency (`queries-compose-filters`) and the reviewer's
narrative unit; per-type tagging would force every story to adopt whole vocabularies. An
empty view is a finding, not an error — declared stories may precede their edges.

## Satisfy

`Engine` (declares views, tags edges at instantiation, extends by restatement, protects
the preset). `Query` (view filters keep tagged edges plus the nodes they relate).

- test — semantics::views_extend_by_restatement
- test — semantics::applications_belong_to_the_views_of_the_edges_they_route
- test — errors::e_stdlib_protected_on_tagging_a_preset_edge
- test — semantics::empty_views_and_uninstantiated_types_are_findings
