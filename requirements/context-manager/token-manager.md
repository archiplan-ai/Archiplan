# TokenManager [PORT]

The TokenManager handles the **form** of a token: it packs `(tokenId, secret)`
into the carrier value a client holds, and unpacks a carrier back into
`(tokenId, secret)`. Just the form — the token itself (its record, grants, and
validity) is created and verified by the **Auth layer**
([auth-layer.md](auth-layer.md)).

- **pack** — `(tokenId, secret)` → the carrier the client carries.
- **unpack** — a client's carrier → `(tokenId, secret)`.

It is a **port**: a core trait with a swappable adapter, so the carrier's shape
can vary. The default shape is `<tokenId>.<secret>`. A customer can swap in
another shape — for example a JWT — when their clients or infrastructure expect
one; the adapter packs and unpacks that shape. Either way the CM creates the token
and verifies it by its secret.
