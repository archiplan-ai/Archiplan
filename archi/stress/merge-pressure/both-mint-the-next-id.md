---
affects: [Archive, StressDoc, Cli]
outcome: breaking
---

# Both mint the next id

Two writers each run a full round against v0005 on their own branch — session, answers, save —
and each save mints `v0006`. The branches merge. Exercised for real: the collision, the naive
repairs, and the remint that versioning.md promises the later writer.

## Attractor

The merged tree carries two claimants for one version id; the second writer repairs by
improvisation; the round records stop being true.

## Resolution

Broke, in layers. The kernel works as promised: the conflict lands only in `archi/versions/`
(`index.toml` content, `v0006.arch.patch` add/add) while both writers' model and session work
merges clean — and once the first-lander's `v0006` is kept, a plain `version save` re-mints the
later writer's answers as the next version, whose stored patch is exactly his semantic delta.
Everything around the kernel is unanswered. Mid-conflict `check` reads the markers as a corrupt
archive and cascades — one collision, `E_ARCHIVE` plus an `E_SESSION` for every session in the
repo, no mention of the recipe. The instinctive union of an append-only manifest is TOML
garbage (`duplicate key`), so the dense-id guard never even speaks. The remint is archaeology:
the note is retyped from the discarded entry, and the save closes no session — the later
writer's round was already stamped. Worst, silently: his session still reads `closed: v0006`,
which now names the *winner's* version; check validates only that the id exists, so the durable
round record lies under a green check, and the one repair — editing `closed:` — is exactly the
hand edit the lifecycle rules forbid. A save is also three artifacts travelling separately: a
`commit -a` ships the manifest without the untracked patch file, leaving the author green
forever and every other clone at `E_ARCHIVE: cannot read` a file the manifest swears by.
Derived: remint-rejoins-the-lineage.
