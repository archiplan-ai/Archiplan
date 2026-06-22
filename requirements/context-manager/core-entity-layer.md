# Core entities layer — Identity

The **fixed core entities**, shipped in core and the same for every customer:
accounts, namespaces, projects, membership, access, tokens.

They are built through the **entity utility** (a core crate) — which produces a
GraphQL entity already wired with a Guard and bound to the Store — so every
identity entity carries a Guard by construction and passes through authorization.

The entity model itself — fields, relations, and rules — is in
[core-data-model.md](core-data-model.md).
