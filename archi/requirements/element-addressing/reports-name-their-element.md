---
kind: functional
origin: intent
satisfied-by: [Addressing]
deferred:
---

# Reports name their element

Every diagnostic, finding, and rendered report line carries the id of the element it concerns — a
node path, a requirement slug, or canonical edge text — in both the human and the `--json` surface.
A reader who sees a wrong or broken line can quote its id back, and the id resolves through `query`
or `search` to exactly one element, so a bug report addresses the system instead of paraphrasing it.

## System Context

The ids already exist inside the tool: resolution works in absolute paths, findings know the element
they fired on, requirements carry slugs. What is missing is carrying them out to the surfaces a
human reads. This is the id-addressability slice `element-addressing` names as the small, testable
first step; the broader read surfaces — requirements as a graph, stress sessions as tables — enter
as their own intents later, each depending on this one. An edge- or port-level id leans on
[[satisfaction-names-the-interface]] to resolve back to its element.

## Satisfy

`Addressing` (stamps every emitted finding and report line with its element's id; the stamped id
round-trips through `query`/`search` back to the one element it names).

- test — addressing: every finding in a fixture check carries a resolvable element id in both the human and the json surface
- test — addressing: a stamped id round-trips through `query` to exactly the element the finding fired on
- test — addressing: a report line about an edge carries the edge's canonical surface text as its id
