---
kind: functional
origin: stressor(a-tampered-or-torn-asset-walks-in)
satisfied-by: [Updater]
deferred:
---

# assets verify by checksum

Before any swap, the downloaded tarball's hash must equal the `.sha256`
the feed publishes beside it. A mismatch — torn wire, stale cache,
lying mirror — is a refusal that names both hashes and leaves the
standing binary untouched. A feed serving no checksum refuses the same
way: unverifiable is not installable.

## System Context

The atomic rename guarantees whole-or-nothing, not right-or-wrong — it
would faithfully install garbage. The installer already verifies the
published checksum; the verbs hold the same bar so no path into
`~/.local/bin` skips it.

## Satisfy

`Updater.fetch` downloads the asset and its `.sha256`, hashes the bytes
through system plumbing (`shasum -a 256` / `sha256sum`), and hands
`Updater.swap` only a verified file.

- test — a fixture feed whose checksum file lies: update refuses naming
  the mismatch, the binary stays byte-identical; the honest checksum
  passes and swaps
