---
affects: [Links]
outcome: breaking
---

# Prose counts as dark delta

A repository keeps its working prose next to the code — issue files, a skills doc, a README —
and the loop's own conventions write to it every round: shortcomings get filed in `issues/`,
resolved issues gain Resolution sections. Every such commit lands hunks the delta scan sees.

## Attractor

Replayed from `issues/audit-counts-repo-prose-as-dark-delta.md` and reproduced on this
repository at its worst right after the fifth round: `archi link audit` reported eight
`unaccounted delta` findings — every one of them `issues/*.md` or `skills/archi.md`, zero code.
The scan excludes `archi/` and `.arch` sources but nothing else that is not code, and linking
markdown to spec elements is not what code-links mean, so the findings are unanswerable: they
persist until the next anchor moves the baseline, the wall grows every round, and the operator
learns to skim past `unaccounted delta` — the one finding the ratchet needs kept loud.

## Resolution

Broke, as filed. Answered this round by letting the project draw the boundary: `[audit]
exclude = ["*.md", "notes/", "exact/path"]` in `archi.toml` — directory prefixes, extension
globs, exact paths — consulted by every link-layer tree scan (audit delta, capture wave,
candidate search), with `archi/`, `.arch` and the manifest staying built-in. This repository
excludes `*.md`; the audit's prose wall is gone and the code delta is fully claimed. Derived:
dark-deltas-are-code.
