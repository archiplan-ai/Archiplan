//! The definition attach pass — `Compiler.Definitions` in the model
//! (`archi/requirements/element-definitions/`): pairs the comments the lexer
//! captured with the elements they define, normalizes and validates the
//! prose, and stores it on the AST between parse and resolution.
//!
//! A definition is the trailing comment on a defining line (`def node`,
//! `def view`, `def rel`, `def conn`, `port`) or the standalone comment
//! block abutting that line from above. A blank line detaches a block;
//! `open` lines, edges and applications take nothing; a whitespace-only
//! comment is no definition; a trailing comment and a block at once is
//! `E_DEFINITION` — one definition site per element. Every violation is a
//! located diagnostic, and the pass reports all of a file's violations in
//! one sweep.

use std::collections::BTreeMap;

use crate::definition;

use super::ast::{BlockItem, DefNodeAst, FileAst, Item};
use super::lexer::Comment;
use super::span::{Diagnostic, SourceMap, Span};

/// Attach every definition comment in the file to its element, collecting
/// violations into `diags`.
pub(crate) fn attach(
    ast: &mut FileAst,
    comments: &[Comment],
    map: &SourceMap,
    diags: &mut Vec<Diagnostic>,
) {
    let pass = Attach {
        trailing: comments
            .iter()
            .filter(|c| c.trailing)
            .map(|c| (c.line, c))
            .collect(),
        standalone: comments
            .iter()
            .filter(|c| !c.trailing)
            .map(|c| (c.line, c))
            .collect(),
        map,
    };
    for item in &mut ast.items {
        pass.item(item, diags);
    }
}

struct Attach<'a> {
    /// Line → the comment trailing that line's code.
    trailing: BTreeMap<usize, &'a Comment>,
    /// Line → the comment-only line sitting there.
    standalone: BTreeMap<usize, &'a Comment>,
    map: &'a SourceMap,
}

impl Attach<'_> {
    fn item(&self, item: &mut Item, diags: &mut Vec<Diagnostic>) {
        match item {
            Item::DefNode(d) => self.def_node(d, diags),
            Item::DefView { name, doc } => *doc = self.definition_at(name.span, diags),
            Item::DefRel { name, doc, .. } => *doc = self.definition_at(name.span, diags),
            Item::DefConn { name, doc, .. } => *doc = self.definition_at(name.span, diags),
            Item::Open(o) => self.block(&mut o.body, diags),
            Item::Edge(_) | Item::App(_) => {}
        }
    }

    fn def_node(&self, d: &mut DefNodeAst, diags: &mut Vec<Diagnostic>) {
        d.doc = self.definition_at(d.path.span, diags);
        self.block(&mut d.body, diags);
    }

    fn block(&self, body: &mut [BlockItem], diags: &mut Vec<Diagnostic>) {
        for item in body {
            match item {
                BlockItem::Port(p) => p.doc = self.definition_at(p.name.span, diags),
                BlockItem::DefNode(d) => self.def_node(d, diags),
                BlockItem::Open(o) => self.block(&mut o.body, diags),
                BlockItem::Edge(_) | BlockItem::App(_) => {}
            }
        }
    }

    /// The definition claimed by the element defined at `span`'s line: its
    /// trailing comment, or the standalone block abutting the line from
    /// above. `None` when neither claims it (or the claim was invalid and
    /// reported).
    fn definition_at(&self, span: Span, diags: &mut Vec<Diagnostic>) -> Option<String> {
        let (_, line, _) = self.map.location(span);
        let line = line as usize;
        let trailing = self.trailing.get(&line).copied();
        let mut first = line;
        while first > 1 && self.standalone.contains_key(&(first - 1)) {
            first -= 1;
        }
        let block: Vec<&Comment> = (first..line).map(|l| self.standalone[&l]).collect();
        let block_text = definition::normalize(
            &block
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
                .join(" "),
        );
        let trailing_text = trailing
            .map(|c| definition::normalize(&c.text))
            .unwrap_or_default();
        let (text, at) = match (trailing_text.is_empty(), block_text.is_empty()) {
            (false, false) => {
                diags.push(Diagnostic::new(
                    "E_DEFINITION",
                    "both a trailing comment and the block above claim this definition — keep one",
                    trailing.expect("trailing text is nonempty").span,
                ));
                return None;
            }
            (false, true) => (trailing_text, trailing.expect("text is nonempty").span),
            (true, false) => {
                let first = block.first().expect("block text is nonempty");
                let last = block.last().expect("block text is nonempty");
                (block_text, first.span.to(last.span))
            }
            (true, true) => return None,
        };
        match definition::validate(&text) {
            Ok(()) => Some(text),
            Err(rule) => {
                diags.push(Diagnostic::new("E_DEFINITION", rule, at));
                None
            }
        }
    }
}
