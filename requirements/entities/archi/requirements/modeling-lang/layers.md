# Epistatic & epistemic layers

Epistatic layer describes the actual structure of the system. Epistemic layer
describes knowledge about the structure: types of components, groups of
elements, etc. Each node belongs to either the epistatic or the epistemic layer.

## Layers & type_of

Layers are distinguished based on `type_of` relation from [stdlib](./modeling-lang.md#standard-library)

- **Term** - nodes that never appear on the left side of the `type_of` relation — a node in the **epistatic** layer: a concrete structural element of the system (a component, a service, a data path, …). Terms are the instances the architecture is actually built from. (Stressor affects, for example, are "epistatic pressure surfaces" — i.e. terms.)
- **Type** - nodes that appear at least once on the left side of the `type_of` relation — a node in the **epistemic** layer that *classifies* terms. A type is what a term is an instance of (e.g. the epistatic terms `EmailService` and `PaymentService` are both classified by the epistemic type `Service`). `Type` is one kind of epistemic node; the epistemic layer also holds other "about the spec" constructs such as groups.
