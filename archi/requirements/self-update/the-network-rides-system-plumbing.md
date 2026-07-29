---
kind: non-functional
origin: intent
satisfied-by: [Updater]
deferred:
---

# the network rides system plumbing

Every byte of HTTP goes through the system `curl` binary and every
unpack through system `tar` — the plumbing doctrine that already carries
git. No HTTP stack, no TLS code, no new runtime dependency enters the
binary.

## System Context

"archi never fetches" was consciously broken once: `git push` inside the
landing verb. `update` and `check-update` join that same exception —
network happens only inside the two explicit verbs, never ambiently; a
machine without `curl` gets a refusal naming what to install.

## Satisfy

Every `Updater` port shells out through `Command` — the same pattern as
the git plumbing in the seats module; the base URL honors
`ARCHI_BASE_URL`, so tests ride `file://` fixtures and never the wire.

- test — the e2e suite runs with no network: `ARCHI_BASE_URL=file:///…`
  serves version and tarball fixtures from disk
