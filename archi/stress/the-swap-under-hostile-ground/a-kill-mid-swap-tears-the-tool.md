---
affects: [Updater]
outcome: surviving
---

# a kill mid-swap tears the tool

Kill -9 the process at every step of `update`: mid-download, mid-unpack,
between unpack and rename. The tool that updates the tool must never
leave a half-written binary at the standing path.

## Attractor

A torn `archi` that cannot run `archi update` again — the repair channel
is the broken thing. The user's only way back is the installer, which
they no longer remember.

## Resolution

Holds by construction: download and unpack happen entirely in a scratch
dir; the standing path is touched exactly once, by `rename(2)`, which
the filesystem gives atomically — the old binary or the new one, never
bytes in between. A kill before the rename leaves the standing binary
untouched and the scratch dir garbage; a kill after is a completed
update. The e2e poisoned-tarball case pins the "before" half.
