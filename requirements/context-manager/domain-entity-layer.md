# Domain entities layer

The customer's context model — **generated**, the domain edge. Each domain entity
carries its type and grants (the allow-model) and binds to the Store.

Domain entities are built through the same **entity utility** as identity, so each
carries a Guard by construction. They form a **generated crate** (the domain edge)
that depends on the core trait crates; the application lists it alongside core
identity and recompiles the binary.

The principle that keeps the core fixed while domain entities attach, and how the
generated edge is trusted, are in
[crates.md](crates.md#generate-and-attach--the-core-stays-fixed) and
[crates.md](crates.md#trusting-the-generated-edges).
