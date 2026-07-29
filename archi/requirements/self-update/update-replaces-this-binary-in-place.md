---
kind: functional
origin: intent
satisfied-by: [Updater]
deferred:
---

# update replaces this binary in place

`archi update` downloads the latest platform tarball and swaps the
running binary atomically. An equal version converges — "already up to
date", exit 0. A torn download, a bad unpack or a failed swap leaves the
standing binary exactly as it was.

## System Context

The installer's platform map is the contract: linux-x64, linux-arm64,
macos-arm64; the tarball unpacks to `archi-<version>-<platform>/archi`.
Windows refuses toward `irm …/install.ps1 | iex` — a running exe cannot
replace itself there, and the refusal names the continuation.

## Satisfy

`Updater.fetch` pulls the tarball into a scratch dir; `Updater.swap`
renames the unpacked binary over `current_exe` — rename is the atom;
nothing is written to the standing path before the new binary is whole.

- test — against a local base URL: a newer fixture tarball lands and the
  swapped binary answers the new number; an equal version exits 0
  without touching the file; a poisoned tarball leaves the binary intact
