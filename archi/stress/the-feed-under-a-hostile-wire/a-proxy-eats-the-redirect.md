---
affects: [Updater]
outcome: surviving
---

# a proxy eats the redirect

Corporate proxies and copycat mirrors flatten or swallow the
`releases/latest` Location header — the redirect that names the tag
never reaches the client.

## Attractor

check-update answers "unreachable" on networks where the browser opens
GitHub fine; users on exactly the networks that need painless updates
learn to distrust the verb.

## Resolution

Holds by inherited design: the installer already walks this wire — the
redirect is the fast path, and when it yields no tag the client falls
back to the API's `releases/latest` JSON. The same two-step rides in
`Updater.check`; a wire where both die is a genuine outage and refuses
naming the continuation.
