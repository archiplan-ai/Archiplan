# Modeling Language

A substrate to express arbitrary ideas about architecture in a structured way that can be transformed or queried later.

## Free graph

Spec consists of nodes and edges, each node and edge has unique identifier and a slug (human readable identifier; 
id and slug can be one thing, depends on implementation).

### Epistatic & epistemic layers and node types

Epistatic layer describes actual strusture of the system. Epistemic layer describes knowledge about 
the strucutre: types of components, groups of elements, etc.
Each node belongs to either epistatic or epistemic layer. Each node of epistatic layer have optional 
type relationaship with at most one node from epistemic layer (type).

#### Types transitivity

Requirements transition from type nodes to their term nodes.

### Edge Types

Each edge has a type that shapes the edge and defines which types of nodes 
