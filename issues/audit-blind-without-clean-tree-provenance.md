# Audit has no delta source until a version is saved on a clean tree

**Kind:** friction (adoption-time) · found by the self-hosting bootstrap

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
