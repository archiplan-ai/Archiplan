---
affects: [Archive, Links.Grader]
outcome: breaking
---

# Dirty-tree bootstrap

A repository adopts archiplan: model written, versions saved, all of it in one working session on
a tree where every file is new and uncommitted. Provenance is recorded only when the tree is
clean at save time, so v0001 and v0002 both mint without a commit — and the first commit lands
only after the saves (`issues/audit-blind-without-clean-tree-provenance.md`).

## Attractor

The audit's default delta source is the latest version's commit provenance. At adoption — the
moment dark-delta coverage would be most instructive — the audit is blind, and stays blind until
the *next* semantic model change happens to be saved on a clean tree: no way to attach provenance
after committing, no way to re-save an unchanged model. The operator learns to pass `--since` by
folklore or stops trusting the audit on day one.

## Resolution

Broke. v0002 has no recovery path: provenance is decided irrevocably at save time. Answered this
round by `Archive.anchor` — post-hoc provenance under the same guarantee save-time recording
gives (the tree is clean and its render hashes to the version being anchored, so the commit
provably contains the render's sources), a no-op rather than a rewrite when provenance already
exists, and named by the sweep's no-delta-source note as the recovery. Derived:
provenance-anchors-post-hoc.
