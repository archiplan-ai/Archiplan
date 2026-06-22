# Versioning

A version is a snapshot of the whole spec at a point in time — types,
terms, edges, requirements, satisfactions, stressors included. Versions
are fractal's unit of "we agreed the architecture looked like this."

## Capabilities

- **Save a version** with a prose note. Captures the complete current
  state of the active scope's project (the whole `spec.json` snapshot).
- **List versions** — every saved version with its note and metadata.
- **Checkout a version** — restore the spec to that snapshot. Working
  state becomes a mutable copy of the stored snapshot; **scope** and
  other cursors re-resolve against the restored tree (invalid paths fail
  clearly rather than silently).
- **Current** — show which version is currently checked out.

## Interaction with stress sessions (Versioning * Stressing)

Saving a version *closes* the active stress session against that
version. At that moment, the incidence report fires automatically (see
[scoring/incidence.md](scoring/incidence.md)), surfacing cross-layer
coupling, stress hotspots, compound vulnerabilities, and under-stressed
components revealed by the session just completed.

Stress **sessions** are identified by opaque **string** ids in v2 (stable
for origins and tooling). Version saves are the natural checkpoint between
stress rounds: each round can produce a new version with the design changes
that answered the breaking stressors.

## Scope versioning (Versioning * Scopes)

Each scope is versioned independently

## Vertical versioning?

Make whole system versioning vertacal => sub-scope change lead to version change

Whole system version?

--Нужна целостная логика версионирования, что такое версия одного взятого скоупа? Когда она меняется? Изменение внутреннего устройства нод меняет версию внешнего скоупа?
