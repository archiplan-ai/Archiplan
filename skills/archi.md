---
name: archi
description: Drive the archiplan workflow end to end — capture intent, derive requirements, model, stress-harden, version, plan, and implement in waves with code-link capture. Use when architecting or implementing a system with archiplan, greenfield or brownfield.
---

# Archi workflow

Ground rules, always:

- Everything is text. Model = `.arch` sources, requirements and stressors =
  markdown under `archi/`, plan = `plan.json`. Mutate by editing files;
  lifecycle moves only through verbs. Run `archi check` after every editing
  round — errors block, findings are the worklist.
- Never invent references. Requirements name model elements by absolute
  path, stressors pin versions, tasks pin nodes — `check` and `plan verify`
  verify every one; a broken reference is a bug you just created.
- Harden first, execute second. Code is written against a *pinned* version,
  never against a moving spec.

## Greenfield

1. **Init** — `archi.toml` (`[project]` name, preset) plus `src/model.arch`.
   `archi build` must pass before anything else.
2. **Capture intent** — one folder per problem area:
   `archi/requirements/<intent>/<intent>.md`, a name and the problem
   statement in the user's own terms. No solutioning here.
3. **Derive requirements** — one file per claim in the intent folder:
   frontmatter (`kind`, `origin: intent`, `satisfied-by: []`, `deferred:`),
   then `System Context` and `Satisfy`. Leave them open —
   `unsatisfied_requirement` findings are the worklist, not errors.
4. **Draft the model** — nodes, ports, typed edges in `.arch`. As elements
   land, fill each requirement's `satisfied-by`, Satisfy prose, and
   verification bullets (`- test — …`). Loop `archi check` to zero errors;
   `archi nkp` for a landscape sanity read.
5. **Save** — `archi version save -m "<why>"` seals the render.
6. **Stress** — `archi/stress/<session>/<session>.md` pinned to that
   version; one stressor file per pressure (`affects`, `outcome`). Breaking
   stressors demand answers: new requirements (`origin: stressor(…)`) and
   model edits. The next `version save` closes the session and prints the
   incidence report — model changed or not: a behavior-only round closes
   against the version it pressed, no mint, exit 0. Repeat 4–6 until a
   round survives — that version is the hardened spec.
7. **Plan** — `archi plan use <name>` (refuses on an unsaved model — save
   first). `archi plan task add <node>` per node to implement; spec_refs
   and requirements are derived, never retyped. Then edit `plan.json`:
   envelope (`problem`, `technology_stack` with provenance,
   `architecture_summary`, `stack_mapping`), per task `description`,
   `inputs` (dependencies, keyed by producing task), `outputs` (files it
   will write — capture attributes deltas through these), `scenarios`, and
   one verification per matched requirement (read the matches from
   `archi plan verify --json`). Loop `plan verify` to clean.
8. **Execute waves** — `archi plan start`. Per wave: implement each
   in-flight task inside its declared outputs, then `archi plan next` — it
   captures the delta into candidate links and blocks on asserted coverage.
   Review `archi link ls --evidence`, `link confirm` the load-bearing
   candidates, `link rm` the drive-bys (subtractions stick), and re-run
   `plan next`. After the last wave it prints the scenarios: verify them
   end to end, then one more `plan next` → `DONE`.
9. **Steady state** — `archi check` and `archi link verify` in CI;
   `archi link audit` for dark deltas, dark spec, and decayed evidence.

## Brownfield

The system exists: the model is *recovered*, not invented, and code-links
are authored from day one.

1. Init as above, inside the existing repo.
2. Capture the intent of the **change being asked**, not the whole legacy —
   the intent scopes what gets modeled.
3. Recover the model: read the code; model only what the intent touches
   plus its boundaries (neighbors as single nodes). Write requirements for
   observed behavior that must not break, alongside the new asks.
4. Anchor reality: `version save`, commit, `archi version anchor` (a
   bootstrap saves on a dirty tree, so provenance — the audit's delta
   source — needs the post-hoc anchor), then
   `archi link add <element> <file#symbol> --kind indirect` for the
   load-bearing existing code — asserted links make `link verify` and
   `link audit` meaningful immediately. `indirect` by default; `literal`
   only where the exact body is the contract.
5. Stress the recovered model as in greenfield — legacy assumptions are
   the best stressors.
6. Plan and execute as greenfield 7–8. Tasks over existing nodes seed
   their incoming edges — the contracts not to break; declare every file
   you will touch in `outputs` so capture attributes your delta instead of
   reporting leftovers.
7. Audit is the ratchet: `unaccounted_delta` findings mean code moved with
   no architectural account — grow the model where they cluster.

## Failure modes

- `link audit` notes no delta source → the last save happened on a dirty
  tree (every bootstrap does); commit, then `archi version anchor` records
  the commit as the latest version's provenance.
- `plan use` refuses → the model has unsaved changes; `version save` first.
- `plan next` blocked on coverage → not an error, the loop: confirm or
  retire the candidates it just minted, re-run.
- verify notes "no longer resolves at Working" → the spec advanced;
  `plan repin`, then fix the tasks it flags.
- Never hand-edit lifecycle state (`state`, `closed_waves`, latches), the
  version archive, or the link journal — verbs only.

Depth: `requirements/tasks.md`, `requirements/code-link.md`,
`requirements/requirements.md`, `requirements/stressing.md`,
`requirements/versioning.md`.
