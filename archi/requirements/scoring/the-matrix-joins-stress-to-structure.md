---
kind: functional
origin: intent
satisfied-by: [Incidence]
deferred:
---

# The matrix joins stress to structure

Incidence pivots recorded pressure into structure: rows are the stressors in scope,
columns the epistatic terms of the frame — the newest pinned version in scope — and a
cell says whether the term is on the stressor's pressure surface. Every affects entry
expands against its own session's pinned version, reconstructed from the archive; the
expanded terms then join against the frame, and terms the frame no longer knows are
dropped visibly — per stressor, under `scope.dropped`, with the `DROPPED_AFFECTS`
warning — because version drift is data, not noise. Scope defaults to the open session,
else the latest closed; `--session <slug>` picks one, `--since <id>` joins every session
pressing that version or later, `--exclude-pending` drops stressors with no outcome yet.
A stress tree that does not compile refuses analysis: the diagnostics come back instead.

## System Context

The declared edges show designed coupling; the matrix shows observed pressure coupling —
the joins the design never admitted are exactly the interesting ones
(`findings-read-the-matrix` names them). Frames are archived versions, never the live
tree, so analyses over past rounds reproduce bit for bit years later
(`stress-pins-versions`).

## Satisfy

`Incidence` (builds rows from sessions, expands per pin, joins against the frame, keeps
drops visible, honors the scope flags).

- test — incidence::affects_expand_against_the_pinned_version_not_the_live_tree
- test — incidence::since_joins_sessions_against_the_newest_frame_and_keeps_drops_visible
- test — incidence::selection_and_validation_speak_plainly
