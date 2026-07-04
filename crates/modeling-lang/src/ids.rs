//! Opaque, stable element identifiers.
//!
//! Every element gets an id at creation from a single per-model counter. Ids are
//! immutable and never reused; everything that must survive edits (edge ends,
//! patterns, delegations) binds to ids, never to names.

macro_rules! id_type {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub struct $name(pub(crate) u64);

        impl $name {
            /// The raw numeric value of this id.
            pub fn raw(self) -> u64 {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "#{}", self.0)
            }
        }
    };
}

id_type!(
    /// Identity of a node.
    NodeId
);
id_type!(
    /// Identity of a port on a node.
    PortId
);
id_type!(
    /// Identity of an edge (relation, connection or application).
    EdgeId
);
id_type!(
    /// Identity of a relation type declared with `rel`.
    RelId
);
id_type!(
    /// Identity of a connection type declared with `conn`.
    ConnId
);
id_type!(
    /// Identity of a view.
    ViewId
);
