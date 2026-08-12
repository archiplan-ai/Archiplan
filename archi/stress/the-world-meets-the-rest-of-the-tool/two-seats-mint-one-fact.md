---
affects: [DocMint, Seats, WorldDoc]
outcome: surviving
---

# Two seats mint one fact

Two worktrees observe the same thing about the world in the same week. Both run `world
add` with titles that slug the same way, each check passes on its own branch, and the
merge brings two files to one path. `parallel-seats-mint-one-slug` recorded this shape
for `req add` and it survived there, but the world file carries more: `uses` names other
facts by slug, so a fused file can hold a graph that reaches into slugs which exist only
on one of the two branches, and `covers` can name elements of two different live models.

## Attractor

The merge produces a file that parses and a graph that does not resolve, so the operator
meets the failure as a wall of unresolved references rather than as one collision. The
repair is by hand, in a file that now mixes two authors' facts about the world under one
name — and merging two claims about reality is exactly the edit nobody should make from
the merge seat.

## Resolution

Held, on the mechanism `parallel-seats-mint-one-slug` already established. Two files at
one path is an add/add conflict, which git raises at the merge instead of fusing
silently. The extra reach of the world file does not escape the net: a resolution that
keeps one side leaves the dropped side's slugs named in `uses`, and those are exactly
the unresolved references `the-header-points-three-ways` makes a blocking error. The
operator meets one conflict and then one located error list, not a quiet fusion.
