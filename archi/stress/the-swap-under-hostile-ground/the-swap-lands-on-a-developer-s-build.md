---
affects: [Updater]
outcome: breaking
---

# the swap lands on a developer's build

Run `archi update` where `archi` is not the installed copy: a
`target/debug/archi` inside a checkout, a symlink into a build dir, a
copy on a USB stick. `current_exe` is whatever answered — the swap
happily replaces a build artifact.

## Attractor

The developer "updates", cargo silently rebuilds over the replaced
artifact on the next build, the update evaporates — and the developer
now trusts a version string that lies. The tool drifts toward "update
means nothing on dev machines".

## Resolution

The swap stays honest by naming, not guessing: the report prints the
absolute path it replaced, and a path under a cargo `target/` dir gets
one warning line — replaced a build artifact, the installer owns
`~/.local/bin`. Symlinks resolve to their target before the rename, so
the link itself never becomes the file. Derived
`the-swap-names-its-target`.
