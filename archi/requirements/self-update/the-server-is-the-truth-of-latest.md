---
kind: functional
origin: stressor(the-server-walks-backward)
satisfied-by: [Updater]
deferred:
---

# the server is the truth of latest

The truth of latest is the release feed — the newest GitHub release tag
of the repository. Any difference against the compiled-in number is
drift, in either direction, but the report names which way: a newer
feed is an update, an older one is the feed's rollback. `update`
converges to the feed's number both ways; equal stays "up to date".

## System Context

The feed's latest is what the operator actually supports — a client
that refuses to follow a rollback strands itself on a number the feed
walked away from. Direction-blind convergence with a direction-naming
report keeps the loop finite: one `update` reaches the fixed point. The
feed resolves like the installer does: the `releases/latest` redirect
names the tag, the API is the fallback when a network eats the
redirect; `ARCHI_REPO` points at a fork, `ARCHI_BASE_URL` replaces the
whole feed for tests and mirrors.

## Satisfy

`Updater.check` compares full semver triples, not equality, and words
the drift by direction; `Updater.fetch` asks for whatever number the
feed named.

- test — a fixture feed offering an older number: check-update calls
  it a rollback, update installs it, the binary answers the older number
