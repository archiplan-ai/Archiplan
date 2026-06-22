# Secrets management

The agent reaches private sources (a private repo, a database, an API) that
need credentials. The question this document scopes: **how does the agent reach
those credentials without ever holding them in plaintext?**

## The trust boundary

The non-negotiable requirement:

- The **agent never holds the secret.** Credentials are decrypted only at fetch
  time, inside a credential broker that performs the read on the agent's behalf —
  the raw secret never enters the agent's context or the model's input.
- Credentials are **stored encrypted at rest** in the Log Analyzer and **never
  logged**; they are decrypted only inside the broker, for a single fetch.
- Access is **scoped per source**: a run is told *which single source* it may
  reach, so a compromised prompt cannot pivot to another service's credentials.
- Every reach is **read-only**.

## Where the credential lives

The Log Analyzer **holds the source credentials itself**, encrypted at rest in
its own database, configured per service (see [`database.md`](database.md),
[`sources.md`](sources.md)). At fetch time the broker decrypts the one source's
credential, performs the read, redacts, and returns the result — the credential
never leaves the broker and the agent only ever sees rows/files. Access is scoped
to a single source per fetch.

This is a self-contained model: the Log Analyzer is the owner of its services'
source credentials and does not depend on the Context Manager (or any other
component) to supply them at analysis time.

## Requirement

The agent must reach private sources using credentials it **never sees in
plaintext**, scoped to a single source per fetch, decrypted only inside the
broker — sourced from the Log Analyzer's **own encrypted store**.

## Open questions

- **Encryption-key management.** Envelope encryption vs. a single derived key,
  and where the key-encryption key lives (KMS / env / operator-supplied).
- **Configuration authorization.** Who may set or change a service's source
  credentials (operator access control).
- **Caching.** Whether a decrypted credential may be cached within a single run,
  or must be re-decrypted per fetch.
