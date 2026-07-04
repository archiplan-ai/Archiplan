//! Recursive-descent parser for the statement language.
//!
//! Statements are separated by `;`; the separator is optional after a braced
//! statement (`X { ... }`), before `}`, and at end of input. `#` starts a
//! line comment.

use crate::ast::{EdgeStmt, Path, PatternAst, Span, Stmt, StmtKind};
use crate::error::{ErrorCode, LangError};
use crate::lexer::{Tok, Token, lex, line_at, line_col};

pub(crate) struct Parser<'a> {
    src: &'a str,
    toks: Vec<Token>,
    pos: usize,
}

/// What an identifier-led statement turned out to be.
enum ExprStmt {
    Block { path: Path, stmts: Vec<Stmt> },
    Edge { edge: EdgeStmt, views: Vec<String> },
    PathOnly(Path),
}

/// The content of a parenthesized group: a port name, a carrier path or a
/// pattern — disambiguated by what follows the closing paren.
enum ParenContent {
    Star,
    Path(Path),
    Classified { anchor: Path, rel: String },
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Result<Self, LangError> {
        Ok(Parser {
            src,
            toks: lex(src)?,
            pos: 0,
        })
    }

    fn peek(&self) -> Tok {
        self.toks[self.pos].tok
    }

    fn cur(&self) -> Token {
        self.toks[self.pos]
    }

    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos];
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        t
    }

    fn err_here(&self, expected: &str) -> LangError {
        let t = self.cur();
        let (line, col) = line_col(self.src, t.start);
        let mut e = LangError::new(
            ErrorCode::Parse,
            format!(
                "expected {expected}, found {} at {line}:{col}",
                t.tok.describe()
            ),
        );
        e.subject = line_at(self.src, t.start).to_string();
        e
    }

    fn expect(&mut self, tok: Tok, expected: &str) -> Result<Token, LangError> {
        if self.peek() == tok {
            Ok(self.bump())
        } else {
            Err(self.err_here(expected))
        }
    }

    fn ident(&mut self, expected: &str) -> Result<String, LangError> {
        let t = self.expect(Tok::Ident, expected)?;
        Ok(self.src[t.start..t.end].to_string())
    }

    fn path(&mut self) -> Result<Path, LangError> {
        let mut segs = vec![self.ident("a name")?];
        while self.peek() == Tok::Dot {
            self.bump();
            segs.push(self.ident("a name after `.`")?);
        }
        Ok(Path { segs })
    }

    fn view_list(&mut self) -> Result<Vec<String>, LangError> {
        let mut views = vec![self.ident("a view name")?];
        while self.peek() == Tok::Comma {
            self.bump();
            views.push(self.ident("a view name after `,`")?);
        }
        Ok(views)
    }

    fn opt_in_views(&mut self) -> Result<Vec<String>, LangError> {
        if self.peek() == Tok::KwIn {
            self.bump();
            self.view_list()
        } else {
            Ok(Vec::new())
        }
    }

    /// Next top-level statement, or `None` at end of input. Enforces the
    /// separator rule after the statement.
    pub fn next_stmt(&mut self) -> Result<Option<Stmt>, LangError> {
        while self.peek() == Tok::Semi {
            self.bump();
        }
        if self.peek() == Tok::Eof {
            return Ok(None);
        }
        let stmt = self.parse_stmt()?;
        match self.peek() {
            Tok::Semi | Tok::Eof => {}
            _ if stmt.is_braced() => {}
            _ => return Err(self.err_here("`;` between statements")),
        }
        Ok(Some(stmt))
    }

    /// Statements inside `{ ... }`; consumes the closing brace.
    fn block_stmts(&mut self) -> Result<Vec<Stmt>, LangError> {
        let mut out = Vec::new();
        loop {
            while self.peek() == Tok::Semi {
                self.bump();
            }
            if self.peek() == Tok::RBrace {
                self.bump();
                return Ok(out);
            }
            if self.peek() == Tok::Eof {
                return Err(self.err_here("`}`"));
            }
            let stmt = self.parse_stmt()?;
            let braced = stmt.is_braced();
            out.push(stmt);
            match self.peek() {
                Tok::Semi | Tok::RBrace => {}
                _ if braced => {}
                _ => return Err(self.err_here("`;` between statements")),
            }
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt, LangError> {
        let start = self.cur().start;
        let kind = match self.peek() {
            Tok::KwNode => {
                self.bump();
                let name = self.ident("a node name")?;
                let block = if self.peek() == Tok::LBrace {
                    self.bump();
                    Some(self.block_stmts()?)
                } else {
                    None
                };
                StmtKind::Node { name, block }
            }
            Tok::KwView => {
                self.bump();
                StmtKind::View {
                    name: self.ident("a view name")?,
                }
            }
            Tok::KwRel => {
                self.bump();
                let trans = if self.peek() == Tok::KwTrans {
                    self.bump();
                    true
                } else {
                    false
                };
                let name = self.ident("a relation type name")?;
                self.expect(Tok::Assign, "`:=`")?;
                let src = self.pattern_slot()?;
                let directed = self.arrow()?;
                let dst = self.pattern_slot()?;
                StmtKind::RelDecl {
                    trans,
                    name,
                    src,
                    directed,
                    dst,
                }
            }
            Tok::KwConn => {
                self.bump();
                let name = self.ident("a connection type name")?;
                self.expect(Tok::Assign, "`:=`")?;
                let src = self.pattern_slot()?;
                let (carrier, directed) = match self.peek() {
                    Tok::Arrow | Tok::BiArrow => (None, self.arrow()?),
                    _ => {
                        let carrier = self.pattern_slot()?;
                        (Some(carrier), self.arrow()?)
                    }
                };
                let dst = self.pattern_slot()?;
                StmtKind::ConnDecl {
                    name,
                    src,
                    carrier,
                    directed,
                    dst,
                }
            }
            Tok::KwOpen => {
                self.bump();
                StmtKind::Open { path: self.path()? }
            }
            Tok::KwRename => {
                self.bump();
                let path = self.path()?;
                let new_name = self.ident("the new name")?;
                StmtKind::Rename { path, new_name }
            }
            Tok::KwDelete => {
                self.bump();
                match self.peek() {
                    Tok::KwRel => {
                        self.bump();
                        StmtKind::DeleteRel {
                            name: self.ident("a relation type name")?,
                        }
                    }
                    Tok::KwConn => {
                        self.bump();
                        StmtKind::DeleteConn {
                            name: self.ident("a connection type name")?,
                        }
                    }
                    Tok::KwView => {
                        self.bump();
                        StmtKind::DeleteView {
                            name: self.ident("a view name")?,
                        }
                    }
                    _ => match self.expr_stmt(false, false)? {
                        ExprStmt::PathOnly(path) => StmtKind::DeleteNode { path },
                        ExprStmt::Edge { edge, .. } => StmtKind::DeleteEdge { edge },
                        ExprStmt::Block { .. } => unreachable!("blocks disabled"),
                    },
                }
            }
            Tok::KwUntag => {
                self.bump();
                let edge = match self.expr_stmt(false, false)? {
                    ExprStmt::Edge { edge, .. } => edge,
                    _ => return Err(self.err_here("an edge statement after `untag`")),
                };
                self.expect(Tok::KwIn, "`in` with the views to remove")?;
                let views = self.view_list()?;
                StmtKind::Untag { edge, views }
            }
            Tok::KwPorts => {
                self.bump();
                let path = self.path()?;
                let views = self.opt_in_views()?;
                StmtKind::Ports { path, views }
            }
            Tok::KwCheck => {
                self.bump();
                StmtKind::Check {
                    views: self.opt_in_views()?,
                }
            }
            Tok::KwDump => {
                self.bump();
                StmtKind::Dump {
                    views: self.opt_in_views()?,
                }
            }
            Tok::Ident => match self.expr_stmt(true, true)? {
                ExprStmt::Block { path, stmts } => StmtKind::Block { path, stmts },
                ExprStmt::Edge { edge, views } => StmtKind::Edge { edge, views },
                ExprStmt::PathOnly(_) => {
                    return Err(self.err_here(
                        "a statement (this path is not followed by an edge, `{`, `(` or `=`)",
                    ));
                }
            },
            _ => return Err(self.err_here("a statement")),
        };
        let end = self.toks[self.pos.saturating_sub(1)].end.max(start);
        Ok(Stmt {
            kind,
            span: Span { start, end },
        })
    }

    /// An identifier-led statement: a scope block, a relation/connection
    /// instantiation, an application, or (when the caller allows it) a bare
    /// path. `allow_views` gates the trailing `in <views>` list.
    fn expr_stmt(&mut self, allow_block: bool, allow_views: bool) -> Result<ExprStmt, LangError> {
        let path = self.path()?;
        match self.peek() {
            Tok::LBrace if allow_block => {
                self.bump();
                let stmts = self.block_stmts()?;
                Ok(ExprStmt::Block { path, stmts })
            }
            Tok::Eq => {
                let port =
                    self.single_name(path, "a port name (ports are local to the current scope)")?;
                self.bump();
                let (inner, inner_port) = self.node_and_port()?;
                self.no_views_on_app(allow_views)?;
                Ok(ExprStmt::Edge {
                    edge: EdgeStmt::App {
                        port,
                        qualifier: None,
                        inner,
                        inner_port,
                    },
                    views: Vec::new(),
                })
            }
            Tok::LParen => {
                self.bump();
                let content = self.paren_content()?;
                if self.peek() == Tok::Eq {
                    // qualified application: port(pattern) = Inner(port)
                    self.bump();
                    let port = self
                        .single_name(path, "a port name (ports are local to the current scope)")?;
                    let qualifier = Some(match content {
                        ParenContent::Star => PatternAst::Any,
                        ParenContent::Path(p) => PatternAst::Exact(p),
                        ParenContent::Classified { anchor, rel } => {
                            PatternAst::Classified { anchor, rel }
                        }
                    });
                    let (inner, inner_port) = self.node_and_port()?;
                    self.no_views_on_app(allow_views)?;
                    Ok(ExprStmt::Edge {
                        edge: EdgeStmt::App {
                            port,
                            qualifier,
                            inner,
                            inner_port,
                        },
                        views: Vec::new(),
                    })
                } else {
                    // connection: A(port) conn(carrier?) B(port)
                    let src_port = match content {
                        ParenContent::Path(p) if p.segs.len() == 1 => {
                            p.segs.into_iter().next().unwrap()
                        }
                        _ => return Err(self.err_here("a port name inside `(...)`")),
                    };
                    let conn = self.ident("a connection type name")?;
                    let carrier = if self.peek() == Tok::LParen {
                        self.bump();
                        let c = self.path()?;
                        self.expect(Tok::RParen, "`)` after the carried node")?;
                        Some(c)
                    } else {
                        None
                    };
                    let dst = self.path()?;
                    self.expect(Tok::LParen, "`(` with the target port")?;
                    let dst_port = self.ident("a port name")?;
                    self.expect(Tok::RParen, "`)` after the port name")?;
                    let views = if allow_views {
                        self.opt_in_views()?
                    } else {
                        Vec::new()
                    };
                    Ok(ExprStmt::Edge {
                        edge: EdgeStmt::Conn {
                            src: path,
                            src_port,
                            conn,
                            carrier,
                            dst,
                            dst_port,
                        },
                        views,
                    })
                }
            }
            Tok::Ident => {
                let rel = self.ident("a relation type name")?;
                let dst = self.path()?;
                let views = if allow_views {
                    self.opt_in_views()?
                } else {
                    Vec::new()
                };
                Ok(ExprStmt::Edge {
                    edge: EdgeStmt::Rel {
                        src: path,
                        rel,
                        dst,
                    },
                    views,
                })
            }
            _ => Ok(ExprStmt::PathOnly(path)),
        }
    }

    /// `Inner(port)` — the right-hand side of an application.
    fn node_and_port(&mut self) -> Result<(Path, String), LangError> {
        let inner = self.path()?;
        self.expect(Tok::LParen, "`(` with the inner port name")?;
        let port = self.ident("a port name")?;
        self.expect(Tok::RParen, "`)` after the port name")?;
        Ok((inner, port))
    }

    fn no_views_on_app(&mut self, allow_views: bool) -> Result<(), LangError> {
        if allow_views && self.peek() == Tok::KwIn {
            return Err(self.err_here(
                "no `in` here — applications are untagged plumbing and belong to the views of the edges they route",
            ));
        }
        Ok(())
    }

    fn single_name(&self, path: Path, expected: &str) -> Result<String, LangError> {
        if path.segs.len() == 1 {
            Ok(path.segs.into_iter().next().unwrap())
        } else {
            Err(self.err_here(expected))
        }
    }

    fn arrow(&mut self) -> Result<bool, LangError> {
        match self.peek() {
            Tok::Arrow => {
                self.bump();
                Ok(true)
            }
            Tok::BiArrow => {
                self.bump();
                Ok(false)
            }
            _ => Err(self.err_here("`->` or `<->`")),
        }
    }

    /// A shape slot: `*` or a parenthesized pattern.
    fn pattern_slot(&mut self) -> Result<PatternAst, LangError> {
        match self.peek() {
            Tok::Star => {
                self.bump();
                Ok(PatternAst::Any)
            }
            Tok::LParen => {
                self.bump();
                let content = self.paren_content()?;
                Ok(match content {
                    ParenContent::Star => PatternAst::Any,
                    ParenContent::Path(p) => PatternAst::Exact(p),
                    ParenContent::Classified { anchor, rel } => {
                        PatternAst::Classified { anchor, rel }
                    }
                })
            }
            _ => Err(self.err_here("a pattern: `*`, `(Node)` or `(Node relName *)`")),
        }
    }

    /// Content of a parenthesized group up to and including the `)`.
    fn paren_content(&mut self) -> Result<ParenContent, LangError> {
        if self.peek() == Tok::Star {
            self.bump();
            self.expect(Tok::RParen, "`)`")?;
            return Ok(ParenContent::Star);
        }
        let anchor = self.path()?;
        match self.peek() {
            Tok::RParen => {
                self.bump();
                Ok(ParenContent::Path(anchor))
            }
            Tok::Ident => {
                let rel = self.ident("a relation type name")?;
                self.expect(Tok::Star, "`*` after the relation name")?;
                self.expect(Tok::RParen, "`)`")?;
                Ok(ParenContent::Classified { anchor, rel })
            }
            _ => Err(self.err_here("`)` or a relation name")),
        }
    }

    /// The source text of a span, trimmed.
    pub fn snippet(&self, span: Span) -> &str {
        self.src[span.start..span.end].trim()
    }
}
