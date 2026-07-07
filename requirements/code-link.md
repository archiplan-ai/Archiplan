# Code-links (spec ↔ code traceability)

A code-link records that some **code** realizes some **spec element** — a **`SpecRef`**: node id or typed
edge, in a scope and version slot (a pinned `vNNNN` or **Working**, the live tree). It is the
machine-checkable bridge between the typed spec and the files and symbols that implement it: during
implementation, agents and teams lose that thread; code-links keep it, and **`archi link verify`** says
whether the graph still matches the tree.

A link carries two layers with opposite mutability:

- the **birth record** — an immutable provenance fact: *these spans, in this delta, were written to
  realize this spec element, under this task*. It is anchored to a delta, not to the moving tree, so it
  never drifts.
- the **projection** — where that code lives *now*: the resolved symbol and its hashes. Derived, and
  recomputed by `verify` — never maintained by hand.

The pin-only model (file + symbol + one content hash) made the link itself the thing that rotted: every
edit anywhere in the body read as drift, verify cried wolf, and ritual re-pinning laundered real drift.
Splitting fact from projection relocates the fragility into a computation.

## Orthogonal surfaces

**`Requirement` / `Sat`** is obligation vs. evidence of satisfaction against **model elements**
([requirements.md](requirements.md#satisfy)); **`archi search`** is natural-language **retrieval**;
**`archi query`** is **structural** graph inspection. None of those tie specific source code to a
specific spec element with drift semantics — code-links do.

## Anchors

The **resolution unit is the symbol**: the item's path within its file (`Type::method` — enclosing
`mod`/`impl`/`trait` names, then the item), which survives formatting, reordering and, verbatim moves
being candidate-tracked, file moves. **Spans appear only in birth records**: they say
where code was born, not where it lives. Non-symbol assets (configs, migrations, schemas) anchor by file
path, optionally with a span.

A projection carries **two hashes** over the anchored item, following the full/interface split of scope
versioning ([versioning.md](versioning.md#versioning--scopes)):

- **interface hash** — the symbol's declared shape: name, signature, visibility; for a type, its public
  surface;
- **body hash** — the whole item, hashed over the **canonicalized token stream**, not source bytes.
  Formatting and comment churn never register — canonical bytes differ iff the code differs, exactly as
  for the model.

Both hashes pin the **canonicalizer version** that produced them.

An **in-code annotation** (`#[archi::realizes("…")]`) is an opt-in third anchor for load-bearing literal
links: it is the one anchor git carries with the code through any refactor, at the cost of coupling the
source file to archiplan. Never the default.

## Kinds and standing

**Kind** says which hash the link watches. A **`literal`** link claims the exact body realizes the spec
element — body-hash drift is signal. An **`indirect`** link claims the symbol's *role* — only
interface-hash drift is signal; internals may churn freely.

**Standing** says what the link may do. An **asserted** link is a claim: it participates in gates and
verifies strictly. An **evidence** link informs — audit, incidence, search ranking — and carries a
**confidence** that accrues when tasks carrying the same spec_ref touch the same symbol, and decays as
the symbol is rewritten without reconfirmation. Evidence never gates ([why](#why-this-shape)).

**Origin** is provenance, orthogonal to standing (as requirement origins are to placement,
[requirements.md](requirements.md#origin)):

| origin | minted by | standing at birth |
|--------|-----------|-------------------|
| `authored` | `archi link add` | asserted, by construction |
| `captured(task)` | task-close capture ([below](#code-links--tasks)) | evidence; `archi link confirm` raises it to asserted — a decision, recorded |

## Code-links * Tasks

The plan is the join point: work happens inside a task that already carries `spec_refs` before the first
line is written ([tasks.md](tasks.md)), so intent is known at write time and never has to be recovered
after the fact — the delta arrives pre-attributed.

When a wave opens, the tree state is recorded as a canonical item-hash index (file → symbol → body
hash — no git involved, so squashes and shallow clones cannot break it). When the wave closes at
`archi plan next`, each task's delta against that index is **captured**: the changed symbols are read
off the index directly, and the task's `spec_refs` × touched symbols become candidate links. The closing agent reviews the batch — subtracts drive-by edits,
asserts the load-bearing links, leaves the rest as evidence. Only then does the coverage gate count
asserted links: the step that demands links is the step that produces them.

Wave discipline does the attribution: tasks with disjoint in-flight windows claim their hunks
unambiguously; overlapping tasks split confidence. `archi link capture --task <TASK>` re-runs a capture
by hand.

## Stored as files

```
archi/links/
  journal.jsonl        # append-only events: add, confirm, repin, retire, touch, decay
```

The journal is the truth; the current link set is its fold: `add` mints (capture emits a batch of
adds), `confirm` asserts, `repin` rewrites a projection, `retire` tombstones; `touch` and `decay` are
capture's confidence observations on evidence links — recorded as events at the moment they are seen,
so confidence itself stays derived, never stored. Birth records store
**content, not references**: file, span, span-content hash, the symbol resolved at capture,
canonicalizer version. A
commit sha is optional provenance — never a dependency, for the same reasons versions refuse git as a
store ([versioning.md](versioning.md#why-this-shape)): squash merges and rewrites orphan shas, shallow
clones cannot resolve them. Projections are recomputed by `verify` and may be cached, never authored.

## Verify and drift

`archi link verify [--spec <SPEC_REF>] [--since <REV>]` recomputes every projection in scope and grades
it:

| state | meaning | asserted | evidence |
|-------|---------|----------|----------|
| **Clean** | anchor resolves; watched hash matches | — | confidence holds |
| **Drifted** | anchor resolves; watched hash moved | review whether spec or code is authoritative; re-pin or fix | confidence decays |
| **Moved** | anchor gone; heuristic candidate elsewhere | `repin --to` the candidate → projection rewritten, birth record untouched | auto-follow at reduced confidence |
| **Missing** | nothing resolves | **Broken**: restore code or retire the link | reported decayed; `audit --prune` retires it |
| **CanonicalizerMismatch** | stored canonicalizer ≠ verifier's | rehash and re-pin; do not ignore | rehash |

The kind picks the watched hash, so an `indirect` link whose body moved while its interface held is
**Clean** — that is the point of the kind. CI exit codes: **Missing** and **CanonicalizerMismatch** fail;
**Drifted** fails only on asserted `literal` links; evidence states never fail.

**Spec-side drift** is the mirror case. A SpecRef resolves at its pinned version by construction — the
archive is sealed — but may not resolve at Working. Because versions are reconstructable and their diffs
semantic ([versioning.md](versioning.md)), the rename or removal that orphaned the ref is locatable in
the version chain: migration is mechanical, or the link retires with the element.

## Audit — dark deltas, dark spec

With deltas as the input, coverage inverts from "which links exist" to "what is unaccounted for".
`archi link audit` reports, advisory like all findings
([errors vs findings](modeling-lang/errors.md#errors-vs-findings)):

| finding | meaning |
|---------|---------|
| `unaccounted_delta` | a hunk since the last version claimed by no task and no link — code motion with no architectural account |
| `unlinked_spec_ref` | a spec element in an active plan's scope with no asserted link and no live evidence |
| `decayed_evidence` | an evidence link whose confidence fell below the floor — confirm or retire |

The delta source is the latest version's commit provenance — recorded only on a clean tree — or an
explicit `--since <rev>`; without either, the audit says so instead of guessing. The aggregate view is
the **spec × code incidence matrix**, same shape as the stressor × component matrix of
[scoring/incidence.md](scoring/incidence.md). Link fragility stops being silent rot and becomes a scored
surface: visible until lifted, never blocking.

## Daily use

Most days you only need three commands; the full loop lives in
[`skills/archi.md`](../skills/archi.md). A `<SPEC_REF>` is `<element>[@<version>]` — a node
path or an edge's canonical surface text; a `<CODE_REF>` is `<file>[#<symbol>]`.

```bash
archi link add <SPEC_REF> <CODE_REF> --kind literal    # or: indirect — authored, asserted
archi link ls [--spec <SPEC_REF>] [--evidence]
archi link verify [--spec <SPEC_REF>] [--since <REV>]  # CI gate; exit codes above
```

Around them: **`archi link repin <LINK_ID> [--to <CODE_REF>]`** rewrites a projection — accepting
drift, following a move; **`archi link confirm <LINK_ID>`** asserts an evidence link;
**`archi link rm <LINK_ID>`** retires one (bulk: `--spec <SPEC_REF> --yes`);
**`archi link capture --task <TASK>`** re-runs a task capture by hand — capture normally fires from
`archi plan next` ([above](#code-links--tasks)); and **`archi link audit`** is aggregate hygiene.

## Why this shape

- **Birth records over pins.** A pin asserts a present-tense correspondence and decays with every commit;
  a birth record is a fact about a delta and never drifts. Fragility moves into the projection —
  recomputed, not maintained.
- **Two hashes, canonical.** One byte-hash makes every edit and every reformat read as drift: verify
  cries wolf, re-pinning becomes ritual, and ritual re-pinning launders real drift. Interface vs. body,
  over canonicalized tokens, makes the alarm mean something.
- **Symbols over spans.** A span is where code was born; a symbol is where it lives. Spans stay in the
  birth record.
- **Capture at the plan join.** Traceability that recovers intent after the fact — commit mining,
  hand-maintained matrices — fights ambiguity forever; here the task names its spec_refs before the code
  exists.
- **Evidence never gates.** Capture is automatic and sometimes wrong (drive-by edits); letting it gate
  would launder noise into obligations. Assertion stays a decision, and gates count only assertions.
- **Not git as the store.** Birth content lives in the tree; shas are provenance — shallow clones and
  rewritten history must not break links.
- **Not annotations by default.** They survive any refactor but couple every consumer source file to
  archiplan; opt-in for load-bearing literal links only.

## Cross-references

- [`tasks.md`](tasks.md) — the plan lifecycle that fires capture and the wave gate.
- [`versioning.md`](versioning.md) — the full/interface hash precedent; git as provenance, never a
  dependency.
- [`requirements.md`](requirements.md) — `Requirement`/`Sat`, the obligation surface; its `test`
  verifications are the natural executable complement — checked by running, not hashing.
- [`kb/code-link.md`](../kb/code-link.md) — full design: model, storage, cascade, CLI reference,
  acceptance posture.
- [`skills/archi.md`](../skills/archi.md) — agent workflow: the full loop from intent to waves, when
  to link, failure modes.
- [`kb/tasks/code_link/`](../kb/tasks/code_link/) — implementation task breakdown for the feature.
