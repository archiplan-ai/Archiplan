---
affects: [Updater]
outcome: breaking
---

# a tampered or torn asset walks in

The tarball arrives short (a flaky wire), stale (a cache), or wrong (a
mirror that lies). tar happens to unpack something plausible — and the
swap installs bytes nobody published.

## Attractor

The atomic rename faithfully installs a corrupt or foreign binary: the
integrity guarantee of the swap launders the wire's garbage into
`~/.local/bin`.

## Resolution

The feed publishes a `.sha256` beside every asset and the installer
already verifies it — the verbs must too: fetch the checksum, hash the
tarball through system plumbing, and refuse a mismatch with the
standing binary untouched. Derived `assets-verify-by-checksum`.
