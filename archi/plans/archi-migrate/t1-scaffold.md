---
node: Scaffold
owns: [one-door-migrates-the-standing-project]
facts: [what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1]
---

# t1 — Scaffold

one migrate page, two passes, the orphan report

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`
- `AgentBrief`
- `Scaffold.emit persist(AgentBrief) SourceTree.store`

## Inputs

## Outputs

- skills/archi-migrate.md
- skills/archi-migrate-world.md
- skills/archi-migrate-links.md
- skills/archi-migrate-fractal.md
- skills/archi.md
- crates/archi/src/scaffold.rs
- crates/archi/tests/init_e2e.rs

## Stack

- `skills/archi-migrate.md` is new: the freshness header, then the triage head — two
  measurements (`archi world ls` empty against a standing model; `archi link ls` counting
  the fifth column under `inferred`) and the `.fractal/` pointer to
  `archi-migrate-fractal` — then the two passes, moved whole: the world interview from
  `skills/archi-migrate-world.md`, the journal triage from `skills/archi-migrate-links.md`;
  both source pages are deleted
- the freshness header names `.claude/skills/archi-migrate/SKILL.md`; the pointer line
  under it (retrieval is `archi-search`) rides along like the siblings'
- `skills/archi-migrate-fractal.md` is corrected against today's binary only where its
  text is false, and its closing step sends the imported project to `archi-migrate` for
  the two passes; nothing else in it moves
- `skills/archi.md` names the world skill in the world section and the links skill in
  step 9 — both mentions become `archi-migrate`
- `scaffold.rs`: `SKILLS` ten → nine; `sync-skills` gains the orphan report — an
  installed `.claude/skills/<name>/SKILL.md` whose name the embedded set lacks prints one
  line naming it as orphaned, removes nothing, and the sync still ends in its usual verdict
- `init_e2e.rs`: `SKILL_MIGRATE_WORLD`/`SKILL_MIGRATE_LINKS` consts fold into
  `SKILL_MIGRATE`; `EMBEDDED_SKILLS` ten → nine; `created.len()` 14 → 13; the world-pass
  behaviour tests read the same content at the new path; the pointer guard's exemptions
  stand; a new test drives the orphan report through a real `sync-skills` run

## Verifications

### one-door-migrates-the-standing-project

- test — a fresh init installs `archi-migrate` byte-equal, and installs neither
  `archi-migrate-world` nor `archi-migrate-links`
- test — the embedded `archi-migrate` opens with the two measurements, carries both
  passes, and names `archi-migrate-fractal` as the old client's own page
- test — `sync-skills` on a tree holding an installed skill the binary does not embed
  reports it as orphaned, by name, and removes nothing
- test — the world-pass content the standing tests read survives at the new path
