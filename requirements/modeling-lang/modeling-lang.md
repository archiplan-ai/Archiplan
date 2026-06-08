# Modeling Language

A substrate to express arbitrary architectureы in a structured way that can be transformed or queried later.

## Textual syntax

Constructs can be expressed syntactically.

## Free graph

Spec consists of nodes and edges, each node and edge has unique identifier and a slug (human readable identifier; 
id and slug can be one thing, depends on implementation).

## Edges

### Types

Edges have explicit `type` property, a type is defined by pair (TypeId, Shape).

#### Direction

- `->` - directed
- `<->` - non-directed

#### Transitivity

An edge is either transitive or not:
- `a -> b; b -> c => a -> c`
- `a <-> b; b <-> c => a <-> c`

#### Shape
Shape restricts the set of nodes an edge of that type can connect.
A. Binary shape `* -> *`:
```
node Data;
node Service;
edge_type transitive type_of := * -> *;
edge_type has_sensitive_data := (Service type_of *) -> (Data type_of *)
```

B. Ternary shape `* (*)-> *` 
```
node Message;
edge_type sends_message := (Service type_of *) (Message)-> (Service type_of *)
```

#### Standard Library

Archiplan provides standard library of edge types to enable [scoring](../scoring/)