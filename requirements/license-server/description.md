# License Server (LS)

The commercial control-plane service — the `LS` the telemetry stream points at
in the [system overview](../description.md). It activates installs, issues and
enforces licenses, inventories the fleet, and receives the telemetry / crash
stream. It never touches project content.

## Connection

The **CM is the single point of contact with the LS** — the deployment's
backend talks to the LS for licensing and telemetry, and this is the system's
**only egress**. The channel is `CM → LS`, identical whether we host the
deployment or the customer self-hosts it. Throughout this document "the
deployment" / "the client" of the LS means that CM.

## Owns vs references

| | |
| --- | --- |
| **Owns** | the activation flow, licenses and their state, signed leases, the per-license instance inventory, the telemetry + crash store, the admin surface |
| **References** | accounts (identity lives in [`core-data-model.md`](../context-manager/core-data-model.md) — a license must bind to an account, see open questions); the **CM** (talks to the LS, verifies leases via an embedded public key) |

## Activation

- **Two-step email proof**: request a code → submit the code. The code is
  short, numeric, single-use, short-lived, with a bounded attempt count and a
  constant-time comparison; one outstanding code per email.
- **Idempotent issuance**: an email that already holds a license gets the same
  key back — re-activation never duplicates.
- **Abuse posture**: per-IP and per-email rate limits on both steps;
  disposable-domain blocklist; all failure responses are generic — they never
  reveal whether an email or code exists.
- **PII discipline**: email addresses are masked in logs and response bodies
  everywhere.

## Licenses

- A license is a **stable, high-entropy random key** bound to its holder, with
  an active flag, `revoked_at` + `revoke_reason`, and an operator note.
- **Revocation is authoritative server-side state** — never client state, never
  inferable or overridable on the client.

## Enforcement — signed leases

- The server signs a **lease** (license, instance, expiry) with an asymmetric
  key (Ed25519); the **public key is embedded in the CM** at build. A valid,
  unexpired lease is the CM's proof of entitlement — it works **offline** for
  the remainder of the lease TTL.
- **TTL is short** (reference: 24 h), so the two consequences stay bounded:
  revocation propagates within one TTL, and offline grace equals one TTL. The
  client refreshes opportunistically well before expiry.
- **Forgery is impossible without the signing key**: clients cannot mint
  leases, and a leaked LS database cannot either — the private key is a
  deployment secret held outside the repo and the DB.
- **Key rotation**: clients accept old + new public keys for a transition
  window, then the old key is dropped.

## Client authenticity

LS is outward-facing — the single egress point of an enterprise's closed
contour. Every request to it (heartbeat, events, crashes) must prove **which
organization** it came from, or a third party could attribute its usage to
another org and defraud billing. A bare license key in the request body is a
bearer credential — anyone who sees it can impersonate the org — so it is not
sufficient.

- **Per-deployment keypair, asymmetric (Ed25519).** Each CM deployment
  (SaaS tenant or enterprise install) generates its own keypair **locally**;
  the **private key never leaves the contour**. LS stores only the **public**
  key. Signing is done with the private key, verification with the public key.
- **Binding at activation.** The public key is registered with LS at the
  authenticated activation step (gated by the license), bound to the
  organization. The customer cannot self-assert identity — LS is the authority
  that binds key → org. Re-registration (key rotation) passes the same gate.
- **Every request is signed** over a canonical payload that includes the
  namespace `id` (its deployment-fixed part identifies the org) and the
  `account_id`, plus a monotonic sequence and timestamp. LS looks up the org's
  registered public key, verifies the signature, and rejects on mismatch —
  unsigned / unverifiable events never reach billing.
- **Egress holds only public material.** The most-exposed component stores the
  least-sensitive keys: a leaked LS database yields public keys only and
  forges nothing. The inventory of secrets:

  | Where | Holds | A leak yields |
  | --- | --- | --- |
  | each org's CM | its own telemetry **private** key | forgery of **that org only** (already trusted for its own metering); rotatable via activation |
  | LS database / admin | **public** keys of all orgs | nothing forgeable |
  | LS deployment secret (outside the DB) | the **one** lease-signing private key | forged leases = piracy, not billing fraud; rotated via the two-key window |

- **Shipped in the client:** only the LS **public** key (to verify leases) —
  public, identical across installs, safe to embed. The org's private key is
  **provisioned at activation**, never baked into the build.

This mirrors the lease direction (LS signs, client verifies with the embedded
LS public key) and the CM token model — one scheme everywhere: private signs,
public verifies, each verifier holds only the other side's public keys.

## Heartbeat & fleet inventory

- One authoritative endpoint: the client proves its identity by signature (see
  *Client authenticity*); the server answers active / inactive (+ kill reason)
  and a fresh lease when active. Rate-limited per license.
- Every install registers an **instance** (OS, first/last seen, last version)
  — a per-license fleet inventory for support and rollout visibility.

## Telemetry & crashes

- **Events**: invocation-level usage (kind, subcommand, exit code, duration),
  keyed by instance + license.
- **Crashes**: panic message **sanitized client-side before sending**;
  backtraces deduplicated by hash.
- **Privacy contract**: nothing project-content-bearing leaves the client;
  scrubbing is the client's obligation — the server stores what it receives,
  so the requirement sits on the client side of the wire.

## Admin

- List / search licenses, revoke with a reason, annotate. A revoke is
  reflected in the very next heartbeat; a fully offline client outlives a
  revoke by at most one lease TTL.

## Open questions

- **Metering integrity (the unforceable residual).** Signatures prove an event
  came from org A, but **cannot force org A to send it** — in an environment
  the customer controls (enterprise on-prem), any emitter can be patched out
  or its egress blocked. This is a law, not a design gap. The lever is to make
  what the customer wants depend on what we want; the candidate mix to decide
  between: capacity / seat billing (removes the incentive — nothing to
  under-report against), usage counters ride the mandatory heartbeat with a
  monotonic sequence (gaps become visible; silence stops the product),
  count-at-issuance for anything billed per unit (the vendor counts what it
  mints, the client never holds the meter — costs an online dependency,
  conflicts with air-gapped), and contract + audit rights as the backstop.
  SaaS has no residual (the vendor runs the CM and sees usage directly); it
  exists only for enterprise self-host, and air-gapped installs are the hard
  case where capacity licensing is essentially the only option.
- **License unit for enterprise**: per-account license (the beta model) vs
  per-namespace seats vs a self-host site license. Decides whether a license
  binds to Account or Namespace in the core data model.
- **Activation ↔ account signup**: does completing activation create the
  Account (and its personal namespace, core-data-model rule 1), or bind to an
  existing one? A standalone email-keyed license registry parallel to
  accounts must not survive integration.
- **Self-host / air-gapped**: offline activation and lease issuance without
  the cloud LS.
- **Billing / payments**: none in the beta; integration point unspecified.
- **Telemetry retention** and aggregation policy.
- **Who else enforces**: does tooling also consult license state (refuse runs
  for revoked licenses), or is the CM the only enforcement point?
