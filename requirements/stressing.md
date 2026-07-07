# Stressing

A stress session is a batch of stressors applied against an explicitly named version of the system model. Each
stressor is a hypothesized failure mode, scale concern, regulatory constraint or stakeholder perspective that
presses on the architecture.

Sessions and stressors are **sources**: structured markdown files with their own git history, compiled and
cross-checked on every `archi check` ([compile](#compile)). They are never copied into versions — they *reference*
version ids ([versioning.md](versioning.md)) — and their schema keeps the machine-readable pressure data in
frontmatter and the reasoning in prose, mirroring [requirements.md](requirements.md).

## Fixed model version

A session presses on exactly one version, named in its session file. The pinned version is immutable and
reconstructable bit-for-bit ([versioning.md](versioning.md)), so the session's ground truth cannot move
underneath it: every stressor's affects resolve — and type entries expand — against the terms of *that* version,
however the live tree evolves meanwhile. Analyses over past sessions are therefore reproducible after the fact,
and a later model edit can never orphan an affects list: it never pointed at the live tree.

At most one session is open at a time; saving a version closes it ([lifecycle](#lifecycle)).

## Stressors

A stressor presses one hypothesis into the model:

- **Description** — what presses. Summary paragraph first: it is the search snippet and the table cell.
- **Affects** — the **epistatic pressure surface**: a mandatory, non-empty list of absolute paths, each naming a
  **term** or a **type** of the session's version. Type entries expand to the set of terms the type classifies
  when incidence and related analyses run ([scoring/incidence.md](scoring/incidence.md)).
- **Attractor** — the state the stressor is trying to pull the system into: what "broken" would look like.
- **Outcome** — `pending` until the session decides, then `surviving` or `breaking`.
  - **Breaking** — the architecture bends: the resolution describes the solution, and its actionable form is
    **derived requirements** — new requirements recording this stressor as their origin
    ([requirements.md#origin](requirements.md#origin)) that the next version must satisfy.
  - **Surviving** — the architecture holds: the resolution says why. Affects stand either way — they record
    where pressure was applied, not how it went.

The derivation link lives on the requirement (`origin: stressor(…)`), one side of truth: a stressor's derived
list is a query over requirements, not a stored field. A breaking stressor that no requirement answers is the
`breaking_unanswered` finding.

There is no mutation vocabulary here either ([source-format.md](modeling-lang/source-format.md)): widening an
affects list, recording an outcome, sharpening an attractor are text edits and a recompile; deleting an obsolete
stressor is deleting its file. The one thing an edit cannot do is empty an affects list — that is
`E_AFFECTS_EMPTY`: a stressor that affects nothing is not a stressor.

## Why affects is mandatory

The affects list is the join key that makes the stressor × component incidence matrix possible. Without it, the
cross-layer analyses that surface hidden coupling, hotspots and compound vulnerabilities would have nothing to
pivot on ([scoring/incidence.md](scoring/incidence.md)).

## Stored as files

```
archi/stress/
  auth-hardening/             # one session = one folder
    auth-hardening.md         #   the session: pinned version, charter, open/closed
    credential-stuffing.md    #   one stressor per file
    token-replay.md
```

The folder is the session; the folder-named file anchors it; every other `.md` in the folder is one stressor of
that session — membership is containment, like requirement hierarchy. Filenames are slugs, auto-derived from
names and unique project-wide ([slug.md](slug.md), [requirements.md#slugs](requirements.md#slugs)).

### Session file

```markdown
---
version: v0003
closed:
---

# Auth hardening

First adversarial round over the fresh auth cut: credential handling and session
lifecycle, before the API surface widens.
```

| field | value |
|-------|-------|
| `version` | the version this session presses on; must exist in the archive manifest |
| `closed` | empty while the session is open; stamped by `archi version save` with the id of the version whose save closed it |

The H1 is the session's name; the first paragraph is its **charter** — what this round presses and why now. At
most one session file may have an empty `closed` (`E_SESSION`).

### Stressor file

```markdown
---
affects: [AuthService, AuthService.Storage]
outcome: breaking
---

# Credential stuffing burst

A botnet replays leaked credential pairs at 100× the organic login rate; real users'
logins are collateral.

## Attractor

`AuthService` saturates on hash verification and the login path is effectively down while
the rest of the system idles — an availability cliff behind one choke point.

## Resolution

Rate limiting and hash-cost isolation take the burst off the hot path: derived
`login-rate-limit` and `hash-offload`, which the next version must satisfy.
```

| field | value |
|-------|-------|
| `affects` | non-empty; absolute paths naming terms or types of the session's version |
| `outcome` | `pending` \| `surviving` \| `breaking` |

Sections in fixed order after the summary-first description: `Attractor`, then `Resolution`. Everything is
present in every file, empty allowed as an explicit state
([requirements.md#file-schema](requirements.md#file-schema)) — except that `Resolution` is non-empty **iff** the
outcome is decided: a verdict without its argument, or an argument without a verdict, is `E_DOC`.

## Lifecycle

A stress round:

1. **Open** — create the session folder and file, pinning the latest version.
2. **Press** — add stressor files; widen affects, sharpen attractors, record outcomes as the architecture
   answers or bends. Everything is a text edit; `archi check` continuously validates the session against its
   pinned version.
3. **Answer** — breaking outcomes derive requirements ([requirements.md](requirements.md)); the design answers
   land as `.arch` edits in the live tree.
4. **Close** — `archi version save` mints the version carrying the answers, stamps the session's `closed`, and
   fires the incidence report over the finished round
   ([versioning.md#versioning--stressing](versioning.md#versioning--stressing)).

The round leaves two artifacts side by side: the version's patch file — *what* changed — and the session
folder — *why*. New requirements from breaking outcomes are the next round's obligations, and version saves are
the natural checkpoints between rounds.

Closed sessions are the durable record of pressure applied. They stay checkable forever because their version
reconstructs exactly; re-running an analysis over an old session expands against the terms it actually pressed
on.

## Compile

`archi check` compiles the stress tree under the shared doc catalog of
[requirements.md#compile](requirements.md#compile) — `E_DOC`, `E_SLUG`, `E_DOC_REF`, `E_MODEL_REF`,
`E_PLACEMENT` — where `E_MODEL_REF` for a stressor means: an affects path that does not resolve in *its
session's* version. Open sessions validate on every check (their version reconstructs and compiles —
[versioning.md#compile](versioning.md#compile)); closed sessions re-validate when an analysis runs over them.

Two codes of its own:

| code | raised when |
|------|-------------|
| `E_AFFECTS_EMPTY` | a stressor's affects list is empty — delete the stressor if it is obsolete |
| `E_SESSION` | `version` or `closed` names no archived version, or more than one session is open |

Findings:

| finding | meaning |
|---------|---------|
| `pending_stressor` | a closed session holds a stressor with no outcome |
| `breaking_unanswered` | a breaking stressor that no requirement records as its origin |
| `empty_session` | a session with no stressors |

## Why this shape

- **One stressor per file.** A search hit is the complete pressure story — what pressed, where (affects), toward
  what (attractor), how it went — and parallel stress work lands as parallel files, not merge conflicts.
- **Affects in frontmatter.** The incidence join key is a grep away; nothing sits between the matrix and its
  input but a YAML list.
- **The session folder sits beside the version patch.** Versions record what changed; sessions record why it had
  to. Both are plain text in the same review.
- **Pinned versions.** A session is evidence, and evidence must not move: affects resolve against an immutable,
  hash-verified model — reproducible years later in a shallow clone.
- **Fixed heading schema.** Machines extract by heading; humans read in the order the method thinks: hypothesis,
  surface, attractor, verdict.
