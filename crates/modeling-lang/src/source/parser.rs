//! Recursive-descent parser for `.arch` files: token stream → [`FileAst`].
//!
//! Fail-fast: the first error in a file aborts that file's parse. All
//! context-free constraints live here (imports first, ports only in `def`
//! blocks, carrier-argument shapes, app ends); name resolution does not.

use super::ast::*;
use super::lexer::{Tok, Token, lex};
use super::span::{Diagnostic, FileId, Span, Spanned};

/// Parse one file. `src` must be the text registered for `file` in the map.
pub(crate) fn parse(file: FileId, src: &str) -> Result<FileAst, Diagnostic> {
    let tokens = lex(file, src)?;
    Parser { tokens, pos: 0 }.file()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.tokens[self.pos].tok
    }

    fn peek_span(&self) -> Span {
        self.tokens[self.pos].span
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn eat(&mut self, tok: &Tok) -> bool {
        if self.peek() == tok {
            self.advance();
            true
        } else {
            false
        }
    }

    fn err_here(&self, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new("E_PARSE", msg, self.peek_span())
    }

    fn expect(&mut self, tok: Tok, what: &str) -> Result<Token, Diagnostic> {
        if self.peek() == &tok {
            Ok(self.advance())
        } else {
            Err(self.err_here(format!("expected {what}, found {}", self.peek().describe())))
        }
    }

    fn expect_newline(&mut self) -> Result<(), Diagnostic> {
        self.expect(Tok::Newline, "end of line").map(|_| ())
    }

    fn expect_ident(&mut self, what: &str) -> Result<Spanned<String>, Diagnostic> {
        match self.peek() {
            Tok::Ident(_) => {
                let t = self.advance();
                let Tok::Ident(s) = t.tok else { unreachable!() };
                Ok(Spanned::new(s, t.span))
            }
            k @ (Tok::Import
            | Tok::Def
            | Tok::Open
            | Tok::Node
            | Tok::View
            | Tok::Rel
            | Tok::Conn
            | Tok::Port
            | Tok::Trans
            | Tok::In) => Err(self.err_here(format!(
                "{} is a reserved word and cannot name an element",
                k.describe()
            ))),
            other => Err(self.err_here(format!("expected {what}, found {}", other.describe()))),
        }
    }

    // ---- file structure -----------------------------------------------------

    fn file(mut self) -> Result<FileAst, Diagnostic> {
        let mut ast = FileAst::default();
        while self.peek() == &Tok::Import {
            ast.imports.push(self.import()?);
        }
        loop {
            match self.peek() {
                Tok::Eof => return Ok(ast),
                Tok::Import => {
                    return Err(self.err_here("imports come first, before any definition"));
                }
                Tok::Indent => return Err(self.err_here("unexpected indentation")),
                _ => ast.items.push(self.top_item()?),
            }
        }
    }

    fn import(&mut self) -> Result<ImportAst, Diagnostic> {
        self.expect(Tok::Import, "`import`")?;
        let module = self.path()?;
        let only = if self.eat(&Tok::LParen) {
            let mut names = vec![self.expect_ident("an exported name")?];
            while self.eat(&Tok::Comma) {
                names.push(self.expect_ident("an exported name")?);
            }
            self.expect(Tok::RParen, "`)`")?;
            Some(names)
        } else {
            None
        };
        self.expect_newline()?;
        Ok(ImportAst { module, only })
    }

    fn top_item(&mut self) -> Result<Item, Diagnostic> {
        match self.peek() {
            Tok::Def => self.def_item(),
            Tok::Open => Ok(Item::Open(self.open_block()?)),
            Tok::Ident(_) => match self.edge_or_app(false)? {
                EdgeOrApp::Edge(e) => Ok(Item::Edge(e)),
                EdgeOrApp::App(a) => Ok(Item::App(a)),
            },
            Tok::Port => Err(self.err_here("ports are declared inside a `def node` block")),
            other => Err(self.err_here(format!(
                "expected a definition, an edge or an application, found {}",
                other.describe()
            ))),
        }
    }

    fn def_item(&mut self) -> Result<Item, Diagnostic> {
        self.expect(Tok::Def, "`def`")?;
        match self.peek() {
            Tok::View => {
                self.advance();
                let name = self.expect_ident("a view name")?;
                self.expect_newline()?;
                Ok(Item::DefView { name })
            }
            Tok::Rel => self.def_rel(),
            Tok::Conn => self.def_conn(),
            Tok::Node => Ok(Item::DefNode(self.def_node()?)),
            other => Err(self.err_here(format!(
                "`def` defines a `node`, `view`, `rel` or `conn`, found {}",
                other.describe()
            ))),
        }
    }

    // ---- type definitions -----------------------------------------------------

    fn def_rel(&mut self) -> Result<Item, Diagnostic> {
        let kw = self.expect(Tok::Rel, "`rel`")?;
        let trans = self.eat(&Tok::Trans);
        let name = self.expect_ident("a relation type name")?;
        self.expect(Tok::ColonEq, "`:=`")?;
        let source = self.slot_pattern()?;
        let directed = if self.eat(&Tok::Arrow) {
            true
        } else if self.eat(&Tok::BiArrow) {
            false
        } else {
            return Err(self.err_here("expected `->` or `<->`"));
        };
        let target = self.slot_pattern()?;
        let span = kw.span.to(target.span());
        self.expect_newline()?;
        Ok(Item::DefRel {
            name,
            trans,
            directed,
            source,
            target,
            span,
        })
    }

    fn def_conn(&mut self) -> Result<Item, Diagnostic> {
        let kw = self.expect(Tok::Conn, "`conn`")?;
        let name = self.expect_ident("a connection type name")?;
        self.expect(Tok::ColonEq, "`:=`")?;
        let source = self.slot_pattern()?;
        let (lanes, target) = self.lanes_and_target()?;
        let span = kw.span.to(target.span());
        self.expect_newline()?;
        Ok(Item::DefConn {
            name,
            source,
            lanes,
            target,
            span,
        })
    }

    fn starts_pattern(&self) -> bool {
        matches!(self.peek(), Tok::Star | Tok::LParen | Tok::Ident(_))
    }

    /// Everything between a conn shape's source and the end of line: the
    /// arrow, its lanes, and the target slot. Carrier-vs-target ambiguity is
    /// resolved by lookahead — a pattern followed by another pattern (or a
    /// `,` lane separator) was a carrier; a pattern followed by nothing was
    /// the target.
    fn lanes_and_target(&mut self) -> Result<(LanesAst, PatternAst), Diagnostic> {
        if self.eat(&Tok::BiArrow) {
            let p1 = self.slot_pattern()?;
            if self.starts_pattern() {
                let target = self.slot_pattern()?;
                Ok((
                    LanesAst {
                        directed: false,
                        fwd_carrier: Some(p1),
                        rev_carrier: None,
                    },
                    target,
                ))
            } else {
                Ok((
                    LanesAst {
                        directed: false,
                        fwd_carrier: None,
                        rev_carrier: None,
                    },
                    p1,
                ))
            }
        } else if self.eat(&Tok::Arrow) {
            let fwd_carrier = if self.peek() == &Tok::Comma {
                None
            } else {
                let p1 = self.slot_pattern()?;
                if self.peek() != &Tok::Comma && !self.starts_pattern() {
                    // `src -> dst`: the pattern was the target all along.
                    return Ok((
                        LanesAst {
                            directed: true,
                            fwd_carrier: None,
                            rev_carrier: None,
                        },
                        p1,
                    ));
                }
                Some(p1)
            };
            let rev_carrier = if self.eat(&Tok::Comma) {
                self.expect(Tok::LArrow, "`<-` (the reverse lane)")?;
                Some(self.slot_pattern()?)
            } else {
                None
            };
            let target = self.slot_pattern()?;
            Ok((
                LanesAst {
                    directed: true,
                    fwd_carrier,
                    rev_carrier,
                },
                target,
            ))
        } else {
            Err(self.err_here("expected `->` or `<->`"))
        }
    }

    // ---- nodes and blocks -----------------------------------------------------

    /// `node Path` or `node Path:` + block. The `def` is already consumed.
    fn def_node(&mut self) -> Result<DefNodeAst, Diagnostic> {
        self.expect(Tok::Node, "`node`")?;
        let path = self.path()?;
        let body = if self.eat(&Tok::Colon) {
            self.expect_newline()?;
            self.block(true)?
        } else {
            self.expect_newline()?;
            Vec::new()
        };
        Ok(DefNodeAst { path, body })
    }

    fn open_block(&mut self) -> Result<OpenAst, Diagnostic> {
        self.expect(Tok::Open, "`open`")?;
        let path = self.path()?;
        self.expect(Tok::Colon, "`:` (`open` takes a block)")?;
        self.expect_newline()?;
        let body = self.block(false)?;
        Ok(OpenAst { path, body })
    }

    /// An indented block. `ports_ok` is true directly inside a `def node`
    /// block — the one place a node's interface is declared.
    fn block(&mut self, ports_ok: bool) -> Result<Vec<BlockItem>, Diagnostic> {
        if !self.eat(&Tok::Indent) {
            return Err(self.err_here("a `:` opens a block: expected an indented line"));
        }
        let mut items = Vec::new();
        while !self.eat(&Tok::Dedent) {
            items.push(self.block_item(ports_ok)?);
        }
        Ok(items)
    }

    fn block_item(&mut self, ports_ok: bool) -> Result<BlockItem, Diagnostic> {
        match self.peek() {
            Tok::Port => {
                if !ports_ok {
                    return Err(self.err_here(
                        "ports are declared in the node's `def`, not in an `open` block",
                    ));
                }
                self.advance();
                let name = self.expect_ident("a port name")?;
                self.expect_newline()?;
                Ok(BlockItem::Port(name))
            }
            Tok::Def => {
                self.advance();
                match self.peek() {
                    Tok::Node => Ok(BlockItem::DefNode(self.def_node()?)),
                    _ => Err(self.err_here(
                        "only nodes are defined inside a scope; views, rels and conns are top-level",
                    )),
                }
            }
            Tok::Open => Ok(BlockItem::Open(self.open_block()?)),
            Tok::Ident(_) => Ok(match self.edge_or_app(true)? {
                EdgeOrApp::Edge(e) => BlockItem::Edge(e),
                EdgeOrApp::App(a) => BlockItem::App(a),
            }),
            Tok::Indent => Err(self.err_here("unexpected indentation")),
            other => Err(self.err_here(format!(
                "expected a port, a nested node, an edge or an application, found {}",
                other.describe()
            ))),
        }
    }

    // ---- edges and applications -------------------------------------------------

    fn edge_or_app(&mut self, in_block: bool) -> Result<EdgeOrApp, Diagnostic> {
        let lhs = self.path()?;
        match self.peek() {
            Tok::Eq => self.app(lhs, None, in_block).map(EdgeOrApp::App),
            Tok::LParen => {
                self.advance();
                let route = self.bare_pattern()?;
                self.expect(Tok::RParen, "`)`")?;
                self.app(lhs, Some(route), in_block).map(EdgeOrApp::App)
            }
            Tok::Ident(_) => self.edge(lhs).map(EdgeOrApp::Edge),
            other => Err(self.err_here(format!(
                "expected a type name (an edge) or `=` (an application), found {}",
                other.describe()
            ))),
        }
    }

    fn edge(&mut self, lhs: PathAst) -> Result<EdgeAst, Diagnostic> {
        let type_name = self.expect_ident("a type name")?;
        let carriers = if self.eat(&Tok::LParen) {
            self.carrier_args()?
        } else {
            Vec::new()
        };
        let rhs = self.path()?;
        let views = if self.eat(&Tok::In) {
            let mut vs = vec![self.expect_ident("a view name")?];
            while self.eat(&Tok::Comma) {
                vs.push(self.expect_ident("a view name")?);
            }
            vs
        } else {
            Vec::new()
        };
        let span = lhs.span.to(self.peek_span());
        self.expect_newline()?;
        Ok(EdgeAst {
            lhs,
            type_name,
            carriers,
            rhs,
            views,
            span,
        })
    }

    fn carrier_args(&mut self) -> Result<Vec<CarrierArg>, Diagnostic> {
        let mut args = Vec::new();
        loop {
            let start = self.peek_span();
            let dir = if self.eat(&Tok::Arrow) {
                Some(LaneDir::Fwd)
            } else if self.eat(&Tok::LArrow) {
                Some(LaneDir::Rev)
            } else {
                None
            };
            let path = self.path()?;
            let span = start.to(path.span);
            args.push(CarrierArg { dir, path, span });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(Tok::RParen, "`)`")?;
        match args.as_slice() {
            [_] => {}
            [a, b] => {
                let dirs = (a.dir, b.dir);
                let one_each = matches!(
                    dirs,
                    (Some(LaneDir::Fwd), Some(LaneDir::Rev))
                        | (Some(LaneDir::Rev), Some(LaneDir::Fwd))
                );
                if !one_each {
                    return Err(Diagnostic::new(
                        "E_PARSE",
                        "two carried nodes must tag their lanes: `(->X, <-Y)`",
                        a.span.to(b.span),
                    ));
                }
            }
            more => {
                return Err(Diagnostic::new(
                    "E_PARSE",
                    "a connection edge carries at most two nodes: one per lane",
                    more[0].span.to(more[more.len() - 1].span),
                ));
            }
        }
        Ok(args)
    }

    fn app(
        &mut self,
        outer: PathAst,
        route: Option<PatternAst>,
        in_block: bool,
    ) -> Result<AppAst, Diagnostic> {
        if !in_block && outer.segments.len() < 2 {
            return Err(Diagnostic::new(
                "E_PARSE",
                "a top-level application names the delegating node: `Node.port = Child.port`",
                outer.span,
            ));
        }
        self.expect(Tok::Eq, "`=`")?;
        let inner = self.path()?;
        if inner.segments.len() != 2 {
            return Err(Diagnostic::new(
                "E_PARSE",
                "the inner end of an application is `Child.port`: a direct child and its port",
                inner.span,
            ));
        }
        let span = outer.span.to(inner.span);
        self.expect_newline()?;
        let mut segs = inner.segments.into_iter();
        let inner_node = segs.next().expect("two segments");
        let inner_port = segs.next().expect("two segments");
        Ok(AppAst {
            outer,
            route,
            inner_node,
            inner_port,
            span,
        })
    }

    // ---- paths and patterns -------------------------------------------------

    fn path(&mut self) -> Result<PathAst, Diagnostic> {
        let first = self.expect_ident("a name")?;
        let mut span = first.span;
        let mut segments = vec![first];
        while self.eat(&Tok::Dot) {
            let seg = self.expect_ident("a name after `.`")?;
            span = span.to(seg.span);
            segments.push(seg);
        }
        Ok(PathAst { segments, span })
    }

    /// A pattern in slot position: `*`, a bare node path, or a parenthesized
    /// pattern — parens are required for classified patterns in slots, where
    /// juxtaposition would be ambiguous.
    fn slot_pattern(&mut self) -> Result<PatternAst, Diagnostic> {
        match self.peek() {
            Tok::Star => {
                let t = self.advance();
                Ok(PatternAst::Any { span: t.span })
            }
            Tok::LParen => {
                let open = self.advance();
                let p = self.bare_pattern()?;
                let close = self.expect(Tok::RParen, "`)`")?;
                Ok(match p {
                    PatternAst::Classified { anchor, rel, .. } => PatternAst::Classified {
                        anchor,
                        rel,
                        span: open.span.to(close.span),
                    },
                    other => other,
                })
            }
            Tok::Ident(_) => Ok(PatternAst::Exact { path: self.path()? }),
            other => Err(self.err_here(format!(
                "expected a pattern: `*`, a node path, or `(Anchor rel *)`, found {}",
                other.describe()
            ))),
        }
    }

    /// A pattern in an already-delimited position (inside parens): `*`,
    /// `Node.path`, or `Anchor rel *`.
    fn bare_pattern(&mut self) -> Result<PatternAst, Diagnostic> {
        match self.peek() {
            Tok::Star => {
                let t = self.advance();
                Ok(PatternAst::Any { span: t.span })
            }
            Tok::Ident(_) => {
                let path = self.path()?;
                if matches!(self.peek(), Tok::Ident(_)) {
                    let rel = self.expect_ident("a relation type name")?;
                    let star = self.expect(Tok::Star, "`*` (classified patterns end with `*`)")?;
                    let span = path.span.to(star.span);
                    Ok(PatternAst::Classified {
                        anchor: path,
                        rel,
                        span,
                    })
                } else {
                    Ok(PatternAst::Exact { path })
                }
            }
            other => Err(self.err_here(format!(
                "expected a pattern: `*`, a node path, or `Anchor rel *`, found {}",
                other.describe()
            ))),
        }
    }
}

enum EdgeOrApp {
    Edge(EdgeAst),
    App(AppAst),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::span::SourceMap;

    fn parse_ok(src: &str) -> FileAst {
        let mut map = SourceMap::new();
        let f = map.add_file("test.arch", src);
        match parse(f, src) {
            Ok(ast) => ast,
            Err(d) => panic!("parse failed: {}", d.render(&map)),
        }
    }

    fn parse_err(src: &str) -> Diagnostic {
        let mut map = SourceMap::new();
        let f = map.add_file("test.arch", src);
        parse(f, src).expect_err("must fail")
    }

    #[test]
    fn node_def_with_ports() {
        let ast = parse_ok("def node AuthService:\n  port handle_login\n  port send_audit_log\n");
        let [Item::DefNode(n)] = ast.items.as_slice() else {
            panic!("one def node");
        };
        assert_eq!(n.path.render(), "AuthService");
        let ports: Vec<_> = n
            .body
            .iter()
            .map(|i| match i {
                BlockItem::Port(p) => p.value.clone(),
                other => panic!("expected ports, got {other:?}"),
            })
            .collect();
        assert_eq!(ports, ["handle_login", "send_audit_log"]);
    }

    #[test]
    fn portless_node_needs_no_block() {
        let ast = parse_ok("def node LoginForm\ndef node Orders.RefundHandler\n");
        assert_eq!(ast.items.len(), 2);
        let Item::DefNode(n) = &ast.items[1] else {
            panic!()
        };
        assert_eq!(n.path.render(), "Orders.RefundHandler");
        assert!(n.body.is_empty());
    }

    #[test]
    fn open_blocks_nest_defs_edges_and_apps() {
        let src = "open AuthService:\n  def node Storage:\n    port save_cred_hash\n  LoginHandler.persist store(->CredHash) Storage.save_cred_hash\n  handle_login = LoginHandler.handle\n";
        let ast = parse_ok(src);
        let [Item::Open(o)] = ast.items.as_slice() else {
            panic!("one open block");
        };
        assert_eq!(o.path.render(), "AuthService");
        assert!(matches!(
            o.body.as_slice(),
            [BlockItem::DefNode(_), BlockItem::Edge(_), BlockItem::App(_)]
        ));
    }

    #[test]
    fn ports_are_rejected_in_open_blocks() {
        let d = parse_err("open A:\n  port x\n");
        assert!(d.message.contains("`open`"), "{}", d.message);
    }

    #[test]
    fn type_defs_are_top_level_only() {
        let d = parse_err("def node A:\n  def view v\n");
        assert!(d.message.contains("top-level"), "{}", d.message);
    }

    #[test]
    fn all_conn_lane_forms_parse() {
        let cases: &[(&str, bool, bool, bool)] = &[
            // (lanes source, directed, has fwd carrier, has rev carrier)
            ("def conn c := * -> *", true, false, false),
            ("def conn c := * ->P *", true, true, false),
            ("def conn c := * ->P, <-Q *", true, true, true),
            ("def conn c := * ->, <-Q *", true, false, true),
            ("def conn c := * <-> *", false, false, false),
            ("def conn c := * <->P *", false, true, false),
            (
                "def conn c := (S type_of *) ->(M type_of *) (S type_of *)",
                true,
                true,
                false,
            ),
        ];
        for (src, directed, fwd, rev) in cases {
            let ast = parse_ok(&format!("{src}\n"));
            let [Item::DefConn { lanes, .. }] = ast.items.as_slice() else {
                panic!("one def conn in {src}");
            };
            assert_eq!(lanes.directed, *directed, "{src}");
            assert_eq!(lanes.fwd_carrier.is_some(), *fwd, "{src}");
            assert_eq!(lanes.rev_carrier.is_some(), *rev, "{src}");
        }
    }

    #[test]
    fn conn_lane_lookahead_separates_carrier_from_target() {
        // `->X Y`: X is the carrier, Y the target. `->X` alone: X is the target.
        let ast = parse_ok("def conn a := * ->LoginForm AuthTarget\n");
        let [Item::DefConn { lanes, target, .. }] = ast.items.as_slice() else {
            panic!()
        };
        assert!(
            matches!(&lanes.fwd_carrier, Some(PatternAst::Exact { path }) if path.render() == "LoginForm")
        );
        assert!(matches!(target, PatternAst::Exact { path } if path.render() == "AuthTarget"));

        let ast = parse_ok("def conn b := * -> AuthTarget\n");
        let [Item::DefConn { lanes, target, .. }] = ast.items.as_slice() else {
            panic!()
        };
        assert!(lanes.fwd_carrier.is_none());
        assert!(matches!(target, PatternAst::Exact { path } if path.render() == "AuthTarget"));
    }

    #[test]
    fn rel_defs_parse() {
        let ast = parse_ok(
            "def rel trans of_sort := * -> *\ndef rel has_pii := (Service type_of *) -> (Data type_of *)\n",
        );
        let Item::DefRel {
            trans, directed, ..
        } = &ast.items[0]
        else {
            panic!()
        };
        assert!(*trans && *directed);
        let Item::DefRel { source, .. } = &ast.items[1] else {
            panic!()
        };
        assert!(matches!(source, PatternAst::Classified { .. }));
    }

    #[test]
    fn edges_parse_with_carriers_and_views() {
        let src = "UI.login login AuthService.handle_login in login_flow\n\
                   A.out send(OrderCreated) B.inbox\n\
                   A.req rpc(->Query, <-Result) B.serve in a, b\n\
                   Service type_of AuthService\n";
        let ast = parse_ok(src);
        assert_eq!(ast.items.len(), 4);
        let Item::Edge(e) = &ast.items[0] else {
            panic!()
        };
        assert_eq!(e.type_name.value, "login");
        assert!(e.carriers.is_empty());
        assert_eq!(e.views.len(), 1);
        let Item::Edge(e) = &ast.items[1] else {
            panic!()
        };
        assert_eq!(e.carriers.len(), 1);
        assert_eq!(e.carriers[0].dir, None);
        let Item::Edge(e) = &ast.items[2] else {
            panic!()
        };
        assert_eq!(e.carriers.len(), 2);
        assert_eq!(e.carriers[0].dir, Some(LaneDir::Fwd));
        assert_eq!(e.carriers[1].dir, Some(LaneDir::Rev));
        assert_eq!(e.views.len(), 2);
        let Item::Edge(e) = &ast.items[3] else {
            panic!()
        };
        assert_eq!(e.lhs.render(), "Service");
        assert_eq!(e.rhs.render(), "AuthService");
    }

    #[test]
    fn two_carriers_must_tag_their_lanes() {
        let d = parse_err("A.p rpc(Query, Result) B.q\n");
        assert!(d.message.contains("tag"), "{}", d.message);
    }

    #[test]
    fn apps_parse_in_flat_and_block_form() {
        let src = "AuthService.handle_login = LoginHandler.handle\n\
                   def node Orders:\n  port events\n  events(OrderCreated) = OrderHandler.handle\n";
        let ast = parse_ok(src);
        let Item::App(a) = &ast.items[0] else {
            panic!()
        };
        assert_eq!(a.outer.render(), "AuthService.handle_login");
        assert!(a.route.is_none());
        let Item::DefNode(n) = &ast.items[1] else {
            panic!()
        };
        let BlockItem::App(a) = &n.body[1] else {
            panic!()
        };
        assert_eq!(a.outer.render(), "events");
        assert!(
            matches!(&a.route, Some(PatternAst::Exact { path }) if path.render() == "OrderCreated")
        );
        assert_eq!(
            (a.inner_node.value.as_str(), a.inner_port.value.as_str()),
            ("OrderHandler", "handle")
        );
    }

    #[test]
    fn top_level_apps_name_the_delegating_node() {
        let d = parse_err("handle = X.y\n");
        assert!(d.message.contains("delegating"), "{}", d.message);
    }

    #[test]
    fn app_inner_end_is_child_dot_port() {
        let d = parse_err("A.p = X.y.z\n");
        assert!(d.message.contains("direct child"), "{}", d.message);
        let d = parse_err("A.p = X\n");
        assert!(d.message.contains("direct child"), "{}", d.message);
    }

    #[test]
    fn imports_parse_and_precede_items() {
        let ast = parse_ok(
            "import auth.service\nimport messages (LoginForm, AuthResponse)\ndef node UI\n",
        );
        assert_eq!(ast.imports.len(), 2);
        assert_eq!(ast.imports[0].module.render(), "auth.service");
        assert_eq!(
            ast.imports[1].only.as_ref().unwrap()[1].value,
            "AuthResponse"
        );
        let d = parse_err("def node UI\nimport auth\n");
        assert!(d.message.contains("imports come first"), "{}", d.message);
    }

    #[test]
    fn reserved_words_cannot_name_elements() {
        let d = parse_err("def node port\n");
        assert!(d.message.contains("reserved"), "{}", d.message);
        let d = parse_err("def node in\n");
        assert!(d.message.contains("reserved"), "{}", d.message);
    }

    #[test]
    fn empty_blocks_are_rejected() {
        let d = parse_err("def node A:\ndef node B\n");
        assert!(d.message.contains("indented"), "{}", d.message);
    }

    #[test]
    fn stray_indentation_is_rejected() {
        let d = parse_err("def node A\n  port x\n");
        assert!(d.message.contains("indentation"), "{}", d.message);
    }

    #[test]
    fn views_clause_only_after_edges() {
        // `in` is reserved: an edge whose rhs would swallow `in` cannot parse it as a name.
        let ast = parse_ok("A dep B in flow\n");
        let Item::Edge(e) = &ast.items[0] else {
            panic!()
        };
        assert_eq!(e.rhs.render(), "B");
        assert_eq!(e.views[0].value, "flow");
    }

    #[test]
    fn error_spans_locate_the_offender() {
        let src = "def node A:\n  port x\n  port 9bad\n";
        let mut map = SourceMap::new();
        let f = map.add_file("m.arch", src);
        let d = parse(f, src).expect_err("must fail");
        let (name, line, _col) = map.location(d.span.unwrap());
        assert_eq!((name, line), ("m.arch", 3));
    }
}
