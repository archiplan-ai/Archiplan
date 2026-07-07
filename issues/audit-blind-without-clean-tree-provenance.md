# Audit has no delta source until a version is saved on a clean tree

**Kind:** friction (adoption-time) · found by the self-hosting bootstrap
**Status:** resolved 2026-07-07 — `archi version anchor`

`archi link audit`'s default delta source is the latest version's commit provenance, and
provenance is recorded only when the git tree is clean at save time
(`versions::provenance`, `crates/archi/src/versions.rs:451`). A bootstrap is the worst case:
every file is new and uncommitted, so v0001 and v0002 both minted without provenance and the
audit can only answer:

    note: no delta source: pass --since <rev>, or save a version on a clean tree
    so its commit provenance anchors the audit

## Impact

Exactly when a repository adopts archiplan — the moment dark-delta coverage would be most
instructive — the audit is blind, and it stays blind until the *next* semantic model change
happens to be saved on a clean tree. There is no way to attach provenance to an existing version
after committing, and no way to re-save an unchanged model (`nothing to save`).

## Fix shape

Any of: accept `archi version save --commit <sha>` / a post-hoc `archi version anchor <id> <sha>`
verb that records provenance for the latest version once the tree is committed; or record HEAD
plus a `dirty` marker at save and let audit use it with a loud caveat. Failing those, `link
audit` could suggest the exact `--since` invocation when the repo has commits.

## Resolution

The post-hoc verb, argument-less: `archi version anchor` (`versions::anchor`,
`crates/archi/src/versions.rs`) records provenance on the version the live render matches, under
the same guarantee save-time recording gives — the tree must be clean and its render must hash to
the version being anchored, so HEAD provably contains the render's sources. Chosen over
`anchor <id> <sha>` because an arbitrary sha cannot be validated cheaply and would admit false
provenance. Recorded provenance is a birth fact: re-anchoring is a no-op that reports it, never a
rewrite. The audit's no-delta-source note now names the recovery (commit, then anchor), and
`requirements/versioning.md#capabilities`, `requirements/code-link.md` and `skills/archi.md`
document the flow. Covered by `versions::tests::anchor_records_provenance_post_hoc`.

Run through the tool's own loop: injected as stressor
`archi/stress/adoption-pressure/dirty-tree-bootstrap.md` (breaking, affects Archive and
Links.Grader) pressing v0002; derived requirement
`archi/requirements/self-hosting/provenance-anchors-post-hoc.md` (`origin:
stressor(dirty-tree-bootstrap)`, satisfied by Archive's `anchor` port); model gained
`Archive.anchor` and its `Cli.drive … Archive.anchor` edge; v0003 closed the round. Delta linked:
l0029 (anchor edge ← `versions.rs#anchor`), l0030 (Archive ← `versions.rs#clean_head`), l0031
(Links.Grader ← `links/mod.rs#audit`), l0032 (Cli ← `main.rs#run_version`).
