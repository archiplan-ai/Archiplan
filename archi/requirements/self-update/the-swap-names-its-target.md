---
kind: functional
origin: stressor(the-swap-lands-on-a-developer-s-build)
satisfied-by: [Updater]
deferred:
---

# the swap names its target

The update report prints the absolute path it replaced. A symlink
resolves to its target before the rename — the link never becomes the
file. A target under a cargo `target/` dir earns one warning line: a
build artifact was replaced, the installer owns `~/.local/bin`.

## System Context

`current_exe` answers with whatever binary is running — the installed
copy, a dev build, a stray copy. The swap must not guess which; it must
say which.

## Satisfy

`Updater.swap` canonicalizes `current_exe`, renames onto the resolved
path, and reports it; the warning fires on a `target/` path component.

- test — the e2e binary lives in a scratch dir behind a symlink: the
  report names the resolved path, the symlink still points there, and
  the file behind it answers the new number
