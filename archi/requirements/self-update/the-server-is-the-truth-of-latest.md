---
kind: functional
origin: stressor(the-server-walks-backward)
satisfied-by: [Updater]
deferred:
---

# the server is the truth of latest

Any difference between the compiled-in number and the server's `latest`
is drift, in either direction — but the report names which way: a newer
server is an update, an older one is the server's rollback. `update`
converges to the server's number both ways; equal stays "up to date".

## System Context

`/download` serves only the current version's tarballs — a client that
refuses to follow a rollback is stranded on artifacts the server no
longer has. Direction-blind convergence with a direction-naming report
keeps the loop finite: one `update` reaches the fixed point.

## Satisfy

`Updater.check` compares full semver triples, not equality, and words
the drift by direction; `Updater.fetch` asks for whatever number the
server named.

- test — a fixture server offering an older number: check-update calls
  it a rollback, update installs it, the binary answers the older number
