# Auth layer

The **Auth layer** does **authentication only**: it turns a raw bearer into a
verified **principal**, and mints carriers at sign-in. Authorization is the
permission layer's job ([permission-layer.md](permission-layer.md)). It uses **two
ports**:

- **TokenManager** — the carrier's shape (mint / parse). See
  [token-manager.md](token-manager.md).
- **the Store Decorator** — to fetch the record by `tokenId` and verify the
  `secret` against the stored `hash(secret)` and validity
  ([store-layer.md](store-layer.md)).

A **token** is a bearer credential a client presents to prove identity — a local
agent or any side tool. Storage shape: the `Token` entity in
[core-data-model.md](core-data-model.md). What a token may *do* — its grants — is
read by PermManager ([permission-layer.md](permission-layer.md)); this document covers the
**authentication** side: minting, the carrier, validation, and validity.

The **carrier** holds only `tokenId` + `secret`. Everything else — the account,
the token's **grants**, the secret hash, and validity — lives in the **store**,
keyed by `tokenId`. The TokenManager handles **shape**, the Auth layer (over the
Store Decorator) handles **trust**.

## Two directions

- **Create (at sign-in).** Given a **payload** — the account and the grants /
  validity for the token — the Auth layer mints `tokenId` + secret, writes the
  record (account, grants, `hash(secret)`, validity) keyed by `tokenId` over the
  Store Decorator, and asks the TokenManager to format the carrier
  ([token-manager.md](token-manager.md)). The full value is shown **once**.
- **Verify (each call).** A call arrives with a raw bearer. The Auth layer asks
  the TokenManager to extract `(tokenId, secret)`, fetches the record by `tokenId`
  over the Store Decorator, compares `hash(secret)`, and checks the token is still
  valid. The `hash(secret)` comparison happens here and the secret is **dropped**;
  the Auth layer emits a thin **principal** — `{ account_id, token_id }`, the
  verified identity. The permission layer takes it from there
  ([permission-layer.md](permission-layer.md)).

## Boundary with the permission layer

The Auth layer authenticates; the **permission layer** authorizes. They stay
decoupled:

- the Auth layer's whole output is the **principal** `{ account_id, token_id }`,
  placed into the request context — the verified identity;
- **PermManager** ([permission-layer.md](permission-layer.md)) takes the principal plus
  the call's `(project, entity type, op)`, reads the token's grants by `token_id`
  from the store — matching its (often many) namespaces and projects — and
  decides;
- they meet only through the principal in the request context (the Guard reads
  it), so each stays independent;
- `hash(secret)` stays inside the Auth layer — verification consumes it, and the
  principal carries identity alone.

## Carrier and validation

- The carrier yields a `tokenId` and a `secret` (its shape is the TokenManager's
  — [token-manager.md](token-manager.md)): the `tokenId` locates the record
  ([core-data-model.md](core-data-model.md)), the `secret` proves possession.
- The store keeps **only `hash(secret)`** — the value itself appears once at
  creation, and the hash is all that remains, so a leaked database exposes hashes
  alone. `secret` is high-entropy (CSPRNG), so a fast unsalted hash suffices and
  validation stays on the hot path.
- Per call: split the value, look up the record by `tokenId`, compare
  `hash(secret)`, then check validity.
- Bearers travel **over TLS only**; the value stays inside the channel, and token
  values stay out of logs and telemetry.
- The first part is `tokenId`, so an account's many tokens (laptop, CI, a specific
  tool) each revoke independently.

## Validity

- Valid = **active and within its lifetime**: `revoked_at` is unset, and
  `duration` is unset or `created_at + duration` is in the future. Checked on
  every use.
- Expiry and revocation **invalidate while keeping the row**, so audit can answer
  "which tokens could touch project X in May".
- **Account deletion cascade-revokes the account's tokens**: `revoked_at` is set
  with `revoke_reason` recording the cause; the same field distinguishes a
  holder's manual revoke from system cascades.
- Removing long-dead rows is background hygiene governed by audit retention.
- **Token status (`active` / `expired` / `revoked`) is derived**, computed by the
  system at read time and returned ready-made through the API; the system judges
  expiry against server time.
- A leaked carrier works for whoever holds it until expiry or revocation, so
  **scope (the grant rows), `duration`, and revocation bound the blast radius** of
  a leaked value.

## Rules

1. **An account manages only its own tokens**: create, list, revoke.
2. **The subject stays the account** (permissions rule 2): actions performed with
   a token are attributed to the account; the `tokenId` is recorded in the audit
   trail. Token values and secrets stay out of logs and telemetry.
