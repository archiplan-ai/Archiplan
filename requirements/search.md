# Search

Given a natural-language phrase, return the ranked list of spec elements 
(nodes, edges, types, requirements, stressors) most related to it, with 
a short snippet for each.

## Results output

slugs (for piping into `archi query`), element cards: scope, version (if saved), 
definition, neighbors' slugs, attached reqs' slugs, stressors' slugs that affect it.

## Working-copy included

Mutations made since the last `version save` are searchable immediately, 
not only after the next snapshot.

## Interaction with versions (Search * Versioning)

Every saved version is independently searchable. `version checkout` shifts the search 
horizon along with the spec.