---
kind: functional
origin: stressor(prose-counts-as-dark-delta)
satisfied-by: [Links]
deferred:
---

# Dark deltas are code

The link layer's tree scans see code and only code: `archi/`, `.arch` sources and the manifest
are excluded built-in, and the project may widen the boundary with `[audit] exclude` in
`archi.toml` — directory prefixes (`issues/`), extension globs (`*.md`), exact paths — consulted
by the audit's delta scan, capture's wave scan and the missing-link candidate search alike, so
candidates and coverage agree on one boundary. Exclusion governs what the scans volunteer, never
what links may claim: a link into an excluded file is added, folded, verified and repinned
exactly like any other.

## System Context

The dark-delta finding is the ratchet's teeth, and prose is where a working repository writes
every day — issue files, skill docs, READMEs. Unanswerable findings train the operator to skim
the one finding that must stay loud. Replayed from
`issues/audit-counts-repo-prose-as-dark-delta.md` at its observed worst: eight findings, all
prose, zero code.

## Satisfy

`Links` (one seam reads `[audit] exclude` from the manifest; `code_files` — capture, candidates,
leftovers — and `delta_hunks` — the audit — both consult it; verify and the fold never do).

- test — links: the audit mutes excluded prose and directory prefixes while an unclaimed code hunk beside them stays dark
- test — links: the excluded walker honors all three pattern forms, and capture's wave scan reads the tree through it
- test — links: a link claiming an excluded file verifies clean — exclusion scopes the scans, not the claims
- test — modeling-lang: the manifest validates `[audit]` strictly — a typo inside the section is an `E_PROJECT`, not a silent no-op
