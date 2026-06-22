# Views

A *view* is one manifestation of the design. Archiplan distinguishes three:

- **Desired** — the architecture as specified (source of truth for intent).
- **Implemented** — the architecture as it exists in the codebase.
- **Deployed** — the architecture currently running in production.

Drift between views is a first-class concern: Archiplan can compare them
and surface gaps (e.g., desired ≠ implemented, or implemented ≠ deployed).