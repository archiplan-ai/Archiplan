//! Statement AST produced by the parser.

use std::fmt;

/// A dot-separated path of names, resolved lexically: the first segment is
/// looked up in the current scope and then outward through enclosing scopes to
/// the root; the remaining segments descend through child scopes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Path {
    pub segs: Vec<String>,
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.segs.join("."))
    }
}

/// A pattern as written: `*`, `(Path)` or `(Path relName *)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PatternAst {
    Any,
    Exact(Path),
    Classified { anchor: Path, rel: String },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Span {
    pub start: usize,
    pub end: usize,
}

/// An edge, addressed structurally by restating it.
#[derive(Clone, Debug)]
pub(crate) enum EdgeStmt {
    Rel {
        src: Path,
        rel: String,
        dst: Path,
    },
    Conn {
        src: Path,
        src_port: String,
        conn: String,
        carrier: Option<Path>,
        dst: Path,
        dst_port: String,
    },
    App {
        port: String,
        qualifier: Option<PatternAst>,
        inner: Path,
        inner_port: String,
    },
}

#[derive(Clone, Debug)]
pub(crate) enum StmtKind {
    Node {
        name: String,
        block: Option<Vec<Stmt>>,
    },
    View {
        name: String,
    },
    RelDecl {
        trans: bool,
        name: String,
        src: PatternAst,
        directed: bool,
        dst: PatternAst,
    },
    ConnDecl {
        name: String,
        src: PatternAst,
        carrier: Option<PatternAst>,
        directed: bool,
        dst: PatternAst,
    },
    Edge {
        edge: EdgeStmt,
        views: Vec<String>,
    },
    Open {
        path: Path,
    },
    Block {
        path: Path,
        stmts: Vec<Stmt>,
    },
    Rename {
        path: Path,
        new_name: String,
    },
    DeleteNode {
        path: Path,
    },
    DeleteEdge {
        edge: EdgeStmt,
    },
    DeleteRel {
        name: String,
    },
    DeleteConn {
        name: String,
    },
    DeleteView {
        name: String,
    },
    Untag {
        edge: EdgeStmt,
        views: Vec<String>,
    },
    Ports {
        path: Path,
        views: Vec<String>,
    },
    Check {
        views: Vec<String>,
    },
    Dump {
        views: Vec<String>,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    /// Whether the statement ends with a closing brace, which makes the `;`
    /// separator optional after it.
    pub fn is_braced(&self) -> bool {
        matches!(
            self.kind,
            StmtKind::Block { .. } | StmtKind::Node { block: Some(_), .. }
        )
    }
}
