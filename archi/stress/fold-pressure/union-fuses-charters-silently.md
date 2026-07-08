---
affects: [StressDoc, SourceTree]
outcome: breaking
---

# Union fuses charters silently

A project that watched `merge=union` heal the link journal applies the same attribute to
`archi/stress/`, expecting fused session files to arrive loud. Two writers open a same-slug
round on their own branches and merge.

## Attractor

The merge boundary that made the journal safe makes the round record unreviewable: the fusion
commits itself.

## Resolution

Broke, and falsified the design it was meant to serve. With identical frontmatter and H1 on
both sides, git merges the common lines clean and the union driver concatenates only the two
charter paragraphs: the result is a schema-perfect session file whose charter is two writers'
sentences stacked — no conflict, no pause, `check` exit 0, merge made by ort. Union is
strictly worse than the default here: markers at least leave a human a reason to stop. The
lesson the round keeps: a session file's identity is its charter prose, and prose fuses
silently under any line-level merge — detection has to be tool-side, reading the boundary git
already draws (the conflict markers), not a smoother merge driver erasing it.
