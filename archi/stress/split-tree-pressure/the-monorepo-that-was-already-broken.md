---
affects: [Links.Grader, Archive, Members]
outcome: breaking
---

# The monorepo that was already broken

A project roots its `archi.toml` two directories below the git root — the nesting the CLI docs
bless as the monorepo shape. Git speaks paths relative to *its* root; archi compares them against
paths relative to *the project* root.

## Attractor

Every `--since` diff and every audit hunk arrives prefixed with directories archi never heard of:
set membership silently misses, changed files count as untouched, dark deltas as dark spec. No
error is raised — the two roots simply never meet, and the audit has been quietly wrong in every
nested project since the day it shipped. Members multiply the shapes in which the roots diverge.

## Resolution

Every git consultation resolves the repository's own top level and rebases its output into the
consulted member's frame; home is a member like any other, so the nested project rides the same
fix. Derived: `git-speaks-from-its-own-root`.
