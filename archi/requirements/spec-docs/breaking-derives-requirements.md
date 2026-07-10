---
kind: functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# Breaking derives requirements

A breaking outcome's actionable form is derived requirements: new claims recording the
stressor in their origin, owed to the next version. The derivation lives on one side
only — a stressor's derived list is a query over requirement origins, never a stored
field — and a breaking stressor no requirement answers is the `breaking_unanswered`
finding. The session folder is the round: the folder-named file pins the version and
carries the charter, every other file in the folder is one stressor, membership by
containment. At most one session is open project-wide (`E_SESSION`), the closing save
stamps `closed:` (`unchanged-saves-close-rounds` owns the ceremony), a closed session
still holding an outcome-less stressor is `pending_stressor`, and a session with no
stressors is `empty_session`.

## System Context

Two sides of one join — requirement origins
(`origin-records-why-placement-records-where`) and stressor outcomes — with the truth
stored once, so a rename or deletion cannot leave the pair disagreeing. Merged and fused
round records fold under their own discipline (`rounds-fold-deliberately`).

## Satisfy

`DocsCompiler` (queries origins for each breaking stressor, enforces the single open
session, validates stamps against the archive, and emits the round findings).

- test — docs::session_discipline
- test — docs::a_save_stamps_the_open_session_closed
- test — docs::folded_sections_validate_and_pending_surfaces
