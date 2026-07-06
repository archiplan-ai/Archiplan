//! The spanned AST of one `.arch` file, as parsed — names unresolved, paths
//! relative to their lexical context. Resolution flattens everything to the
//! absolute-path statement layer.

use super::span::{Span, Spanned};

/// A dotted name as written: one or more identifier segments.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct PathAst {
    pub segments: Vec<Spanned<String>>,
    pub span: Span,
}

impl PathAst {
    /// The path as written, `a.b.c`.
    pub fn render(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.value.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }
}

/// A shape or routing pattern as written.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum PatternAst {
    /// `*`
    Any { span: Span },
    /// A node path: matches exactly that node.
    Exact { path: PathAst },
    /// `Anchor rel *` (parenthesized where juxtaposition demands).
    Classified {
        anchor: PathAst,
        rel: Spanned<String>,
        span: Span,
    },
}

impl PatternAst {
    pub fn span(&self) -> Span {
        match self {
            PatternAst::Any { span } => *span,
            PatternAst::Exact { path } => path.span,
            PatternAst::Classified { span, .. } => *span,
        }
    }
}

/// `import a.b` or `import a.b (X, Y)`.
#[derive(Clone, Debug)]
pub(crate) struct ImportAst {
    pub module: PathAst,
    /// `None` imports every export of the module.
    pub only: Option<Vec<Spanned<String>>>,
}

/// The carried lanes of a conn definition. `directed` is `false` only for
/// `<->` shapes, which have at most a single (forward) carried slot.
#[derive(Clone, Debug)]
pub(crate) struct LanesAst {
    pub directed: bool,
    pub fwd_carrier: Option<PatternAst>,
    pub rev_carrier: Option<PatternAst>,
}

/// One carrier argument at a conn edge: `X`, `->X` or `<-X`.
#[derive(Clone, Debug)]
pub(crate) struct CarrierArg {
    pub dir: Option<LaneDir>,
    pub path: PathAst,
    pub span: Span,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum LaneDir {
    Fwd,
    Rev,
}

/// An edge statement: `lhs type[(carriers)] rhs [in views]`. Whether it is a
/// rel edge or a conn edge — and how the end paths split into node and port —
/// is decided at resolution, by the kind of `type_name`.
#[derive(Clone, Debug)]
pub(crate) struct EdgeAst {
    pub lhs: PathAst,
    pub type_name: Spanned<String>,
    pub carriers: Vec<CarrierArg>,
    pub rhs: PathAst,
    pub views: Vec<Spanned<String>>,
    pub span: Span,
}

/// An application: `outer_port[(route)] = Child.port`. The left path's last
/// segment is the port; an empty prefix delegates the enclosing block's node
/// (block form).
#[derive(Clone, Debug)]
pub(crate) struct AppAst {
    pub outer: PathAst,
    pub route: Option<PatternAst>,
    pub inner_node: Spanned<String>,
    pub inner_port: Spanned<String>,
    pub span: Span,
}

/// `def node Path[: block]`.
#[derive(Clone, Debug)]
pub(crate) struct DefNodeAst {
    pub path: PathAst,
    pub body: Vec<BlockItem>,
}

/// `open Path: block` — augment an existing node's scope.
#[derive(Clone, Debug)]
pub(crate) struct OpenAst {
    pub path: PathAst,
    pub body: Vec<BlockItem>,
}

/// One item inside a node block. `Port` is legal only directly inside a
/// `def node` block — the interface lives at the definition.
#[derive(Clone, Debug)]
pub(crate) enum BlockItem {
    Port(Spanned<String>),
    DefNode(DefNodeAst),
    Open(OpenAst),
    Edge(EdgeAst),
    App(AppAst),
}

/// One top-level item.
#[derive(Clone, Debug)]
pub(crate) enum Item {
    DefView {
        name: Spanned<String>,
    },
    DefRel {
        name: Spanned<String>,
        trans: bool,
        directed: bool,
        source: PatternAst,
        target: PatternAst,
        span: Span,
    },
    DefConn {
        name: Spanned<String>,
        source: PatternAst,
        lanes: LanesAst,
        target: PatternAst,
        span: Span,
    },
    DefNode(DefNodeAst),
    Open(OpenAst),
    Edge(EdgeAst),
    App(AppAst),
}

/// One parsed `.arch` file.
#[derive(Clone, Debug, Default)]
pub(crate) struct FileAst {
    pub imports: Vec<ImportAst>,
    pub items: Vec<Item>,
}
