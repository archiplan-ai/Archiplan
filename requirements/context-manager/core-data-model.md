# Core Data Model

The identity, tenancy and access entities — the **fixed (uniform) part** of the
client store, the same schema for every client. They live in the same decorated
store as the context (see [`database.md`](database.md)); the context part keys
its data by `namespace_id` / `project_id` from here.

This doc: who exists (accounts), where work lives (namespaces, projects), and
how access is stored (membership, project access, tokens). The behavioral rules
— who may grant what — live in [`permission-layer.md`](permission-layer.md); how a call
is authenticated and authorized by the CM, in
[`tooling-layer.md`](tooling-layer.md).

## Entities

### Account

| Field | Notes |
| --- | --- |
| `id` | unique identifier |
| `email` | unique |
| `nickname` | display handle — not unique |
| `avatar` | image reference |
| `created_at` | |

### Namespace

| Field | Notes |
| --- | --- |
| `id` | composite: a **deployment-fixed part** (set once in the installation's configuration) + a **random part** |
| `name` | display name — not unique |
| `owner_id` | → Account |
| `personal` | boolean — marks an account's personal namespace |
| `created_at` | |

The deployment-fixed id part makes every namespace id **self-attributing**:
given only the id (in telemetry, support, logs), it is known which
installation — for self-host, which customer organization — the namespace
belongs to. The random part carries uniqueness; the fixed part carries
attribution only and is never a security boundary. Ids are immutable —
changing the configured part affects only namespaces created afterwards.

### Project

| Field | Notes |
| --- | --- |
| `id` | unique identifier |
| `namespace_id` | → Namespace — a project belongs to exactly one namespace |
| `name` | |
| `created_at` | |
| `updated_at` | last modification of the project's content or settings |

### Membership (account ↔ namespace)

| Field | Notes |
| --- | --- |
| `account_id` | → Account |
| `namespace_id` | → Namespace |
| `management` | boolean — namespace-management right (see [`permission-layer.md`](permission-layer.md)) |

Primary key: (`account_id`, `namespace_id`) — one membership row per pair.
Being a manager requires being a member: management is a flag on the
membership row.

### Project access (account ↔ project ↔ entity type)

| Field | Notes |
| --- | --- |
| `account_id` | → Account |
| `project_id` | → Project |
| `entity_type` | the entity type within the project (Tasks, Code, spec, …) |
| `level` | `R` or `RW` |

Primary key: (`account_id`, `project_id`, `entity_type`). Access is **per entity
type** (see [`permission-layer.md`](permission-layer.md)): each row grants one type. The
owner's rights are implicit, held via `owner_id`.

### Token

| Field | Notes |
| --- | --- |
| `id` | token id; the first part of the value `<token_id>.<secret>`, safe to log |
| `account_id` | → Account |
| `name` | human-readable label ("laptop-agent") |
| `token_hash` | hash of the secret — the token value itself is never stored |
| `created_at` | |
| `duration` | optional lifetime; unset = does not expire |
| `revoked_at` | `NULL` = active |
| `revoke_reason` | set with `revoked_at`: holder's action or a system cascade (e.g. account deletion) |

The token's rights are stored relationally in the two tables below and are
kept a subset of the account's rights at all times (write-time cascade): the set
granted at creation stays ⊆ the account's, and narrowing an account right (or
removing the account from a namespace) cascades to the matching token rows.
Carrier format: [`token-manager.md`](token-manager.md). Validity and verification:
[`auth-layer.md`](auth-layer.md).

### Token project access (token ↔ project ↔ entity type)

| Field | Notes |
| --- | --- |
| `token_id` | → Token |
| `project_id` | → Project |
| `entity_type` | the entity type within the project |
| `level` | `R` or `RW` |

Primary key: (`token_id`, `project_id`, `entity_type`). Mirrors Project access;
granted at token creation as a subset of the issuing account's rights.

### Token namespace management (token ↔ namespace)

| Field | Notes |
| --- | --- |
| `token_id` | → Token |
| `namespace_id` | → Namespace |

Primary key: (`token_id`, `namespace_id`). Row presence = the token carries
the namespace-management right. Owner-only operations have no representation
here by construction.

## Rules

1. **Every new account creates its personal namespace.** Created automatically
   at signup with `personal = true` and `owner_id` = the account; the account
   becomes its first member. Exactly one personal namespace per account
   (unique `owner_id` where `personal = true`).
2. **An account can be added to an unlimited number of namespaces.** No cap on
   memberships in either direction: an account joins many namespaces, a
   namespace holds many accounts.
3. **Any account can create a namespace and becomes its owner.** No gate on
   creation; the creator is the `owner_id`.
4. **The personal namespace is fixed** — it cannot be deleted, left, or
   renamed. It exists for the account's lifetime.
5. **Names are not unique.** `nickname` and namespace `name` are display only;
   the `id` is the identifier. No uniqueness constraint, no collision handling.

## Open questions

- **Account deletion** (deferred — a separate question). What happens to the
  personal namespace; to non-personal namespaces the account **owns** (there is
  no ownership transfer, so they would orphan); to the account's memberships and
  project access rows. Tokens are already defined: cascade revoke
  ([`auth-layer.md`](auth-layer.md)).
