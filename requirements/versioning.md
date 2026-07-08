# Versioning

A version is a semantic snapshot of the system model at a point in time — the unit of "we agreed the
architecture looked like this." Stress sessions anchor to versions ([stressing.md](stressing.md)); the
archive of past versions is the durable record of the architecture's evolution.

The live source tree holds exactly one model: the current one. A past version is never a copy of the
source tree — it is the model's **canonical form**, archived under `archi/versions/` as keyframes and
patches, hash-sealed and reconstructable without git. Git history remains provenance and the recovery
path, never a dependency: resolving a version must work in a shallow clone. The source tree is the only
persistence ([source-format.md](modeling-lang/source-format.md)), and the archive is part of the tree.

Requirements, stress sessions and intent are **not** copied into a version (as the spec.json-era
snapshot did) — they are files with their own git history that *reference* version ids.

## Canonical form

```
canonical(project) = render(lower(project))
```

The compiled model rendered back to surface syntax as a single flattened module, statements in the
deterministic lowering order of
[source-format.md](modeling-lang/source-format.md#lowering-and-determinism). Two existing invariants
make this well-defined: identical sources lower to an identical batch bit for bit, and dumps round-trip
(Fidelity). The reserved-word caveat cannot arise: source is the only editing surface, so every archived
model is source-built and round-trips by construction.

Canonicalization is the compression. Comments, formatting and file organization are stripped, so
canonical bytes differ **iff the model differs** — and because statements sit in path/name-sorted order,
a line diff between two renders is a semantic diff: an inserted definition does not shift unrelated
lines.

A version's identity is `sha256` over its canonical bytes. Consequences:

- **Versions mint only on semantic change.** `save` mints nothing when the new render hashes equal to
  the latest version. Comment and formatting churn never creates versions. The no-op save is still a
  success — and still closes an open stress round (see [Versioning * Stressing](#versioning--stressing)).
- **"Current" is derived, not stored.** The live tree is *at* whichever version's hash its render
  matches; otherwise it is dirty relative to the latest version.

## Stored as files

```
archi/versions/
  index.toml            # append-only manifest, one entry per version
  v0001.arch            # keyframe: full canonical render
  v0002.arch.patch      # unified diff from v0001's canonical bytes
  v0003.arch.patch      # ...from v0002's reconstruction
  v0026.arch            # next keyframe
```

Manifest entry:

```toml
[[version]]
id      = "v0042"
note    = "close stress round: payment-degradation"
created = "2026-07-07T12:31:00Z"
model   = "sha256:ab12…"        # hash of the canonical bytes
parent  = "v0041"
kind    = "patch"               # or "full"
commit  = "de3eb58"             # optional provenance; never a dependency
```

- **Ids** are a dense sequence `vNNNN`; `parent` is the previous id — lineage is linear.
- **Keyframe policy.** The first version is a keyframe. A later save writes a keyframe when the patches
  since the last keyframe — including the one this save would write — together exceed the size of the
  new render; otherwise it writes the patch. Total archive bytes therefore stay within about twice the
  keyframe bytes, whatever the churn pattern.
- **Patches** are unified diffs (3 lines of context) against the previous version's canonical bytes.
  They apply mechanically to hash-verified input — the context is for the human reader, not fuzzy
  matching. Reconstructing `vK` = nearest keyframe at or before `K`, apply forward patches, verify
  against `model` in the manifest.
- **The archive is sealed.** Every file's content is pinned by the hash chain in the manifest; editing a
  keyframe, a patch or a manifest entry is a compile error, not a drift. Recovery is git history.
- Keyframes are marked `linguist-generated` in `.gitattributes` so forges collapse them in review.
  Patches stay reviewable on purpose: the patch *is* the change record of the round — a permanent,
  human-readable answer to "what changed in the architecture between v41 and v42."

## Capabilities

- **Save** (`archi version save -m "note"`) — compile the live tree, render canonical, hash. Refuse if
  the hash equals the latest version's. Otherwise write a patch or keyframe per policy and append the
  manifest entry. Saving closes the active stress session (see below). The note is mandatory prose.
  `commit` provenance is recorded only when the working tree is clean at save time — so the commit
  really contains the sources the render came from.
- **Anchor** (`archi version anchor`) — record `commit` provenance post hoc on the version the live
  render matches. A save on a dirty tree mints without provenance — adoption's normal case: a bootstrap
  saves before its first commit — leaving `link audit` no delta source
  ([code-link.md](code-link.md#audit--dark-deltas-dark-spec)). Committing and anchoring closes that gap
  under the save-time guarantee: the tree must be clean and its render must hash to the version being
  anchored. Provenance is a birth fact — anchoring a version that has it is a no-op reporting the
  recorded commit, never a rewrite.
- **List** — every version with note and metadata, from the manifest.
- **Show** — materialize a version's canonical source. The output is compilable source.
- **Diff** — semantic diff between two versions' canonical renders. For adjacent versions this is the
  stored patch, verbatim.
- **Current** — report which version the live render matches, or that the tree is dirty since the
  latest.
- **Restore.** Restoring the live source tree to a past version is a git operation on the source tree —
  the manifest's `commit` field points at the provenance. In a clone without history, `show`'s render
  compiles and can seed a tree, at the cost of the original modular layout and comments.

## Versioning * Stressing

A stress session names the version it presses on, explicitly ([stressing.md](stressing.md)). Saving a
version *closes* the active stress session against the version just saved. At that moment the incidence
report fires automatically ([scoring/incidence.md](scoring/incidence.md)), surfacing cross-layer
coupling, stress hotspots, compound vulnerabilities and under-stressed components revealed by the
session.

A save that finds the model unchanged mints nothing but still finishes the ceremony: it closes the open
session against the *current* version — a round whose answers were code, docs or tests hardens the
version it pressed without re-minting it — fires the same incidence report, and exits 0. With nothing
to close, the no-op save reports and exits 0. Two rounds may therefore close against the same version
id; the sessions' `closed:` fields, not the version lineage, are the record of the rounds. Only genuine
failures — compile diagnostics, a corrupt archive, two open sessions — exit nonzero.

Version saves are the natural checkpoint between stress rounds: a round that changed the model produces
a new version carrying the design changes that answered its breaking stressors, and the round's patch
file is the record of those changes; a behavior-only round closes against the version it pressed, its
record the session file and the code delta.

Because versions are reconstructable, a session's analyses are reproducible after the fact: a
stressor's type-affects expand against the terms of the version the session actually pressed on, not
the current model.

## Versioning * Scopes

Scope versions are **derived** from whole-model versions, not stored independently — one storage
mechanism, no coherence problem between scope and system versioning.

Every node has a Merkle-style hash over its canonical subtree. A scope's version identity is its
subtree hash; its version history is the sequence of distinct subtree hashes across saved versions.
Vertical propagation holds by construction: a change anywhere under a node moves the hashes of all its
ancestors and of no sibling.

Two hashes per scope turn "does an internals change bump the outer scope's version?" into policy rather
than storage:

- **full hash** — everything under the node, in canonical order;
- **interface hash** — the node's declared ports plus its boundary edges (edges with exactly one end
  inside the subtree).

An internals-only change moves the full hash and leaves the interface hash; consumers choose which hash
their notion of "changed" compares. Root-node subtree hashes are recorded per version in the manifest,
so "which versions touched scope X" is a manifest scan; deeper scopes reconstruct-and-hash on demand.

## Versioning * Multiplayer

Two branches that each mint `v0043` collide in the manifest and in filenames as an ordinary git merge
conflict — surfaced, not silent. The later branch re-mints its save on top of the merged lineage.
Concurrent-editing discipline lives in [multiplayer.md](multiplayer.md).

## Compile

On `archi check`:

- reconstruct every archived version and verify it against its manifest hash; a mismatch is an error
  naming the version;
- verify the id sequence is dense and the parent chain linear;
- compile the versions referenced by open stress sessions and validate each stressor's affects against
  *its* version.

Hash verification is cheap text patching; per-version compilation happens only where semantics demand
it.

## Why this shape

- **Not git history as the store.** Shallow CI clones can't resolve it, squash-merges and rewrites
  orphan it, and it would make git a second persistence layer behind the source tree. The `commit`
  field is provenance only.
- **Not compressed binaries.** Git packfiles delta-compress similar text almost to the size of the
  change; a `tar.zst` per version defeats that — every version costs its full size in history forever,
  and can't be diffed, reviewed or merged. In a git repo, compression *is* canonicalization plus plain
  text.
- **Not full snapshots per version.** Working-tree and PR cost grow linearly in model size × version
  count; for large models every save would land a multi-megabyte file in review.
- **Not deltas as truth** (the previous Archiplan). Patches here are a storage encoding under a
  hash-verified interface: the semantic layer never sees them, every version is independently
  verifiable, the current model always exists in full form as the live source, and keyframes bound the
  chains. The manifest's `kind` field keeps encodings per-entry, so they can coexist if this ever needs
  to change.
