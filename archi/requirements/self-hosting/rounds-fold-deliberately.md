---
kind: non-functional
origin: stressor(two-rounds-one-record, markers-pass-for-prose, remint-consumes-the-fused-record, a-fold-crosses-pins)
satisfied-by: [Sessions, DocsCompiler]
deferred:
---

# Rounds fold deliberately

Folding two concurrent stress rounds into one record is a deliberate act with a trace: the
merged state — two open sessions, or one session name claimed by two charters — is detected and
named with its recipe, the fold preserves both charters in the surviving record, and a
session's `closed:` stamp stays true across a remint. A chimera round, one record silently
assembled by a merge, is impossible to produce by accident.

## System Context

A session folder is the durable record of pressure applied; branch-parallel rounds merge as
ordinary files, so today the record can be authored by the merge itself. Pressed for real by
merge-pressure, then by fold-pressure: the union attribute auto-commits the fusion, git's
default markers pass for charter prose (`check` green with the conflict still in the tree),
and the archive's own remint recipe re-stamps a fused seal. The boundary git already draws —
the markers — is the one honest detection surface; nothing reads it.

## Satisfy

`DocsCompiler` reads the boundary git draws: conflict markers anywhere under `archi/stress/`
are one `E_SESSION` naming the file, the claimed session and the fold verb — ahead of any
parse error, the way the manifest's markers already earn a recipe-naming `E_ARCHIVE` — and the
two-open-sessions diagnostic names the same verb. `Sessions` owns the only path that merges
round records: `fold` joins two open folders into one (`--into`) or normalizes a marker-fused
file in place, keeps both charters — the folded round's charter, pin and stamp under a
`## Folded` heading the schema validates forever — and refuses folds across pins and any
sealed record it cannot keep true: a fused sealed pair folds with the surviving stamp intact
and the folded stamp marked for remint, which `Archive.remint` re-stamps — never the stamp
that already tells the lineage's truth. Remint refuses a session whose file still holds
markers, so the sequence archive → fold → remint cannot skip its middle.

- test — markers in a charter, in frontmatter, or in a stressor are one `E_SESSION` naming the session and the fold verb, not prose and not a bare parse error
- test — `session fold <loser> --into <winner>` moves the stressors, keeps both charters under `## Folded`, refuses filename collisions, and the survivor passes `check` forever
- test — a marker-fused open pair folds in place; `--keep` picks the surviving charter and the other lands folded
- test — a fold across differing pins refuses and names the rule
- test — a fused sealed pair folds with stamps preserved; `remint --session` re-stamps the folded stamp and refuses while markers remain
