---
affects: [Scaffold, DocsCompiler, Cli]
outcome: breaking
---

# The standing project has no wing

Upgrade a project that has run for a year into a binary that knows the wing. There is
no `archi/world/` in the tree and there is no fact in it. Every node stands
unconditioned, every behavior is justified from the architecture side alone, and
`check` was green a minute ago. `init` is create-only and this tree is already
initialized, so nothing scaffolds the folder; the first `world add` meets a path that
does not exist.

## Attractor

Either the wing arrives loud — a finding per unconditioned node on a tree that changed
nothing — and the operator mutes the whole class on the first day, or it arrives so
quietly that nobody learns it exists until a year of behavior has been justified
without it. The count that `the-check-counts-the-wing` prints reads zero on a tree
where the wing is absent and on a tree where it was emptied, and those are not the
same state.

## Resolution

The wing arrives silent and announced: derived `the-wing-arrives-without-noise`. No
finding fires for an uncovered node, so a tree that was green stays green and no class
gets muted on the first day. The first `world add` creates the folder, so no verb meets
a missing path. Discovery goes through the briefing, which is how this tool has always
told its agents what exists. The absent-versus-emptied ambiguity in the count stands:
both read as no line, and neither is a state a verb acts on.
