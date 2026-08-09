---
version: v0019
closed: v0019
version-hash: sha256:fc4353b0b02fad4e2c6cba9f4805820941e82195fdebdba1e035d597e5bc5c7e
---

# the base under a moving remote

The mint has always branched from whatever the local checkout happened
to hold, however old. This round presses the fix — refresh the base
before minting — where it can hurt: no network, a local branch that
carries unpushed work, a member checkout in the middle of something,
and the folder the mint lands in when it runs from inside a seat.
