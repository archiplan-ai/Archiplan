# Self-update

The old fractal client is untouchable — it stews in itself, and its server
already tells it a newer number lives at `/version`. The new archi has no
ear at all: a user installs once and never learns that 0.1.11 exists short
of re-running the installer by hand. Wanted, in the user's words: a
`check-update` that says whether a newer version exists, and an `update`
that downloads it and replaces the binary. The server side stands —
`/version` answers `{"latest": …}` and `/download` serves the platform
tarballs — so this intent is client-only.
