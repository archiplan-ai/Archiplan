# Requirements

A requirement is a unit of desirable shape of the software — a claim the architecture must uphold. Requirements
are **sources**: structured markdown files in the project tree, compiled together with the model and
integrity-checked against it on every `archi check` ([compile](#compile)). They are not part of the model's
canonical form and are never copied into versions — they are living documents with their own git history that
reference model elements by path ([versioning.md](versioning.md)).

The model and the requirements have different read surfaces on purpose: agents read the model through the lowered
JSON statements ([agent-interface.md](agent-interface.md)), but they read requirements by reading the files — the
markdown itself is the interface. The schema below fixes what lives where, so a requirement parses
deterministically, greps cleanly and retrieves as a self-contained card ([search.md](search.md)).

## Requirement body

At minimum, a prose description. The first paragraph after the name is the **summary** and must stand alone: it
is what search results, report tables and retrieval snippets show. Detail follows in further paragraphs.

### System Context

The pre-existing landscape the architecture must land onto: external services, mandated technology,
organizational constraints. Context is part of the claim — a requirement satisfied only outside its context is
not satisfied.

## Kinds

`functional` or `non-functional`. The kind is a frontmatter field — analyses that weigh functional load read it
without touching prose ([scoring](scoring/scoring.md)).

## Origin

Every requirement records where it came from:

| origin | meaning |
|--------|---------|
| `intent` | derived directly from the enclosing [intent](intent.md) — the folder the requirement sits in. Intent-origin requirements are not added during a stress session: mid-session requirements answer pressure, not new problem statements. |
| `parent` | a pure refinement of the parent requirement it nests under |
| `stressor(slug, …)` | the answer to one or more breaking [stressors](stressing.md) |
| `fusion(slug, …)` | emerged at the junction of requirements ([fusion](#fusion)) |

The positional kinds are checked against placement: `intent` is legal only at the root of an intent folder,
`parent` only where a parent requirement exists (`E_PLACEMENT`). The slug kinds are checked against existence:
every named slug must resolve (`E_DOC_REF`). Placement and provenance are orthogonal — a stressor-derived
requirement still *lives* somewhere in an intent's tree: its `origin` records why it exists, its path records
what it refines.

## Fusion

A junction of several requirements often produces a new one. Archiplan keeps the relation
`req1 * req2 * … => req'` as the fused requirement's origin: `fusion(rate-limits, tenant-isolation)`.

## Deferred requirements (unchecked)

Some requirements are acknowledged — typically as a reaction to a stressor — but deliberately not addressed by
the current architecture. Deferring is explicit: the `deferred` field carries the reason, and `check` then
reports the info-level `deferred_requirement` finding instead of `unsatisfied_requirement` — deferrals stay
visible, so they expire by being seen, not by being forgotten. An unsatisfied requirement with an empty
`deferred` is *open*: undecided work the next round owes an answer.

## Satisfy

The claim that named elements of the model satisfy the requirement. The `satisfied-by` field lists the elements
as absolute model paths; the `Satisfy` section carries the prose *how*. One satisfaction record per requirement,
inline at the requirement — and it holds together: elements without prose or prose without elements is `E_DOC`.
Un-satisfying a requirement is emptying both.

An entry may name a **type**; the claim then covers every term the type classifies, expanding exactly as
stressor affects do ([scoring/incidence.md](scoring/incidence.md)).

### Transitivity

A satisfied parent satisfies its subrequirements by definition: `check` reports nothing under a satisfied
ancestor.

### Verification

How the satisfaction claim can be proved — trailing bullets of the `Satisfy` section, tagged by variant:

- `test` — describes the test(s) that perform the necessary checks;
- `type-level` — describes how the requirement is enforced at the type level, so a violation cannot compile.

Verifications are subject to scoring: the more requirements carry formulated verifications the better
([scoring](scoring/scoring.md)); a satisfaction record with no verification entries is the
`unverified_satisfaction` finding.

### Satisfy * Agent Interface

There is no satisfy vocabulary to call: an agent satisfies a requirement by filling `satisfied-by` and the
`Satisfy` prose and recompiling, and un-satisfies by emptying them — a text edit, like every other mutation
([source-format.md](modeling-lang/source-format.md)). `archi check` immediately says whether the claim resolves.

## Stored as files

```
archi/requirements/
  secure-auth/                  # one intent = one folder
    secure-auth.md              #   the intent itself: the problem statement (intent.md)
    no-plaintext-credentials.md #   a requirement (file scale)
    session-revocation/         #   an epic requirement (folder scale)
      session-revocation.md     #     the epic's own claim
      revoke-on-breach.md       #     a subrequirement
```

**Containment is the hierarchy.** A requirement nests under what it refines: files at an intent folder's root
are that intent's requirements; files in an epic's folder are its subrequirements. There is no parent field to
keep in sync — the path is the parent pointer.

A requirement exists at one of **three scales**, same schema at each:

- **section** — a heading inside its parent's file: name plus prose, nothing else. It inherits every field from
  the parent — kind, origin (`parent` by construction), deferral, and satisfaction via
  [transitivity](#transitivity);
- **file** — `<slug>.md`: the full schema, own frontmatter;
- **folder** — `<slug>/` with `<slug>.md` inside: a file-scale requirement whose subrequirements outgrew inline
  sections.

The moment a section needs any field of its own — individual satisfaction, a stressor origin, its own deferral —
promote it to a file; when a file's subrequirement sections outgrow comfortable reading, promote it to a folder.
Promotion is a mechanical move that changes no meaning, exactly like splitting a module in
[source-format.md](modeling-lang/source-format.md).

### Slugs

The filename **is** the slug ([slug.md](slug.md)), auto-derived from the name: lowercased, runs of
non-alphanumerics collapsed to `-`. A file whose name does not derive to its filename is `E_SLUG`; section-scale
requirements derive theirs from the heading. Slugs are the reference currency — origins, stress tables, search
results and report cards all speak slugs ([human-interface.md](human-interface.md)) — and they are unique
project-wide across archiplan primitives (intents, requirements, stressors, sessions).

## File schema

```markdown
---
kind: non-functional
origin: intent
satisfied-by: [AuthService.Storage]
deferred:
---

# No plaintext credentials

Credentials never persist in plain text: whatever stores them holds only salted hashes.

Applies to backups and logs as much as to the primary store — "persist" means any byte
that survives the request.

## System Context

Argon2id is the organization-mandated KDF; an HSM fleet is out of scope for this system.

## Satisfy

`AuthService.Storage` is the only node the `store` connection reaches with `CredHash`, and
the hash is produced before persistence — no path writes a raw credential.

- test — register, then scan every storage fixture and log sink for the raw credential
- type-level — `store` carries `CredHash`, not `LoginForm`: a raw write does not compile

## Password rotation keeps sessions

Rotating a password must not invalidate the user's other active sessions.
```

Frontmatter — the machine fields:

| field | value |
|-------|-------|
| `kind` | `functional` \| `non-functional` |
| `origin` | `intent` \| `parent` \| `stressor(slug, …)` \| `fusion(slug, …)` |
| `satisfied-by` | `[]`, or absolute model paths — terms or types ([satisfy](#satisfy)) |
| `deferred` | empty, or the reason this is outside the current architecture's scope |

Body — the prose, in fixed order: the name (H1) and its [summary-first body](#requirement-body), then
`System Context`, then `Satisfy` (prose, then [verification](#verification) bullets), then subrequirement
sections. `System Context` and `Satisfy` are the **reserved headings**; **any other heading opens a
subrequirement** — headings are structure, not decoration. Formatting inside a section is free markdown.

**Every field and reserved section is present in every file.** Empty is an explicit state — an empty
`satisfied-by` says *no satisfaction claimed yet*; a missing field says nothing, so it is `E_DOC`. Absence is
ambiguity, and ambiguity does not compile.

## Compile

`archi check` compiles the requirement tree together with the model and cross-checks them — the documents cannot
silently rot against the source:

- every file parses against the schema: frontmatter fields, reserved sections, section order — all present,
  possibly empty;
- slugs derive from names, match filenames, and are unique project-wide;
- every origin slug resolves, and the origin kind agrees with placement;
- every `satisfied-by` path resolves in the **current** model — rename a node in `.arch` and forget the
  requirement that names it, and the build breaks at that requirement's file and line.

Diagnostics carry `file:line:col` and a stable code, like model compile diagnostics
([source-format.md](modeling-lang/source-format.md#errors)). The doc-source catalog, shared with
[stressing.md](stressing.md):

| code | raised when |
|------|-------------|
| `E_DOC` | a doc source violates its schema: missing or unknown field or section, misordered sections, malformed frontmatter, a half-present satisfaction record |
| `E_SLUG` | a name does not derive to its filename, or two primitives collide on a slug; both sites reported |
| `E_DOC_REF` | a slug reference (an origin's stressor or fusion set) names no existing primitive |
| `E_MODEL_REF` | a model path does not resolve — `satisfied-by` against the live model; stressor affects against their session's version |
| `E_PLACEMENT` | a doc sits where its meaning forbids: a requirement outside an intent folder, `origin: intent` off the folder root, `origin: parent` with no parent, a stressor outside a session |

Findings — advisory, never blocking
([errors vs findings](modeling-lang/errors.md#errors-vs-findings)):

| finding | meaning |
|---------|---------|
| `unsatisfied_requirement` | no satisfaction record, not deferred, no satisfied ancestor — open work |
| `deferred_requirement` | a deferral in force, reported with its reason — visible until lifted |
| `unverified_satisfaction` | a satisfaction record with no verification entries |

Unsatisfied is a finding, not an error, by design: closing a stress round *produces* open requirements
([stressing.md](stressing.md)) — the very save that records them must not be blocked by them. The discipline is
that every requirement is in exactly one declared state: satisfied, deferred with a reason, or visibly open.

## Why this shape

- **One requirement per document node, name and summary first.** A search hit is the whole story — claim,
  context, satisfaction, proof — and its first two lines are already the card ([search.md](search.md)). Sections
  chunk at headings, so a retrieval window never splits a field.
- **Machine fields in frontmatter, prose in sections.** The join keys — origin slugs, satisfied-by paths — are
  plain YAML lists: `check`, indexers and `grep` read them without parsing prose, and prose never has to encode
  data.
- **Containment is the hierarchy.** The path is the parent pointer; there is no back-reference to drift, and
  promotion between scales changes reading ergonomics, not meaning.
- **Markdown is the read surface.** The model's read surface for agents is lowered statements; a requirement's
  is its file. The fixed schema is what makes reading a file reliable rather than hopeful — and the order
  machines extract is the order a human wants: what, where, how, proof.
- **Compile-checked prose.** A requirement that names model elements is a claim the build enforces. Requirements
  are not versioned into the archive — versions pin the model, satisfaction claims track the live tree — so a
  model edit that invalidates a claim surfaces at the next `check`, not at the next audit.
