//! The code side of a code-link: canonicalization and symbol anchors
//! (`requirements/code-link.md#anchors`).
//!
//! A file canonicalizes to a token stream — comments and formatting are
//! stripped, so canonical tokens differ iff the code differs, exactly as
//! canonical model bytes do. Rust files (`rust-tok-v1`) additionally index
//! their **items**: `fn`, `struct`, `enum`, `trait`, `mod`, `const`,
//! `static`, `type`, `union`, `macro_rules!`, addressed by a `::`-joined
//! symbol path (`mod::Type::method` — `impl` and `trait` blocks and bodied
//! `mod`s open scopes). Every other extension canonicalizes generically
//! (`text-v1`): whole-file anchors only.
//!
//! An item carries the two hashes of the spec's full/interface split: the
//! **body hash** over all its tokens, and the **interface hash** over its
//! signature — for a `fn`, the tokens before the body; for every other
//! kind the whole item *is* its interface, and the hashes coincide.

use std::fmt::Write as _;

use sha2::{Digest, Sha256};

/// Canonicalizer for Rust sources: comment-stripping tokenizer + item index.
pub const RUST_CANON: &str = "rust-tok-v1";
/// Canonicalizer for everything else: whitespace-collapsing tokenizer.
pub const TEXT_CANON: &str = "text-v1";

/// The canonicalizer a file's extension selects.
pub fn canonicalizer_of(file: &str) -> &'static str {
    if file.ends_with(".rs") {
        RUST_CANON
    } else {
        TEXT_CANON
    }
}

/// Whether this binary knows a stored canonicalizer — an unknown one is the
/// `CanonicalizerMismatch` verify state, never silently rehashed.
pub fn knows(canonicalizer: &str) -> bool {
    canonicalizer == RUST_CANON || canonicalizer == TEXT_CANON
}

/// One canonical token: its text and the 1-based source line it starts on.
#[derive(Clone, Debug)]
pub struct Token {
    pub text: String,
    pub line: usize,
}

/// An indexed item of a Rust file.
#[derive(Clone, Debug)]
pub struct Item {
    /// `::`-joined symbol path: enclosing `mod`/`impl`/`trait` names, then
    /// the item's own name.
    pub symbol: String,
    /// 1-based line span of the item, attributes included.
    pub start_line: usize,
    pub end_line: usize,
    /// Hash over all the item's canonical tokens.
    pub body: String,
    /// Hash over the signature tokens (`fn` only differs from `body`).
    pub interface: String,
}

/// A canonicalized file: the token stream and, for Rust, its item index.
pub struct Canonical {
    pub canonicalizer: &'static str,
    pub tokens: Vec<Token>,
    pub items: Vec<Item>,
}

impl Canonical {
    /// Hash of the whole file's canonical tokens.
    pub fn file_hash(&self) -> String {
        hash_tokens(&self.tokens)
    }

    /// The items matching a symbol: exact matches, or — when none — items
    /// whose path ends with `::<symbol>`, so `verify` on a leaf name works
    /// until it collides.
    pub fn find(&self, symbol: &str) -> Vec<&Item> {
        let exact: Vec<&Item> = self.items.iter().filter(|i| i.symbol == symbol).collect();
        if !exact.is_empty() {
            return exact;
        }
        let suffix = format!("::{symbol}");
        self.items
            .iter()
            .filter(|i| i.symbol.ends_with(&suffix))
            .collect()
    }
}

/// Canonicalize a file's text under the canonicalizer its name selects.
pub fn canonicalize(file: &str, text: &str) -> Canonical {
    match canonicalizer_of(file) {
        RUST_CANON => {
            let mut tokens = rust_tokens(text);
            drop_trailing_commas(&mut tokens);
            let items = scan_items(&tokens);
            Canonical {
                canonicalizer: RUST_CANON,
                tokens,
                items,
            }
        }
        _ => Canonical {
            canonicalizer: TEXT_CANON,
            tokens: text_tokens(text),
            items: Vec::new(),
        },
    }
}

/// `sha256:<hex>` over the token texts, unit-separated so token boundaries
/// stay part of the identity (`ab c` ≠ `a bc`).
pub fn hash_tokens(tokens: &[Token]) -> String {
    let mut hasher = Sha256::new();
    for t in tokens {
        hasher.update(t.text.as_bytes());
        hasher.update([0x1f]);
    }
    hex_digest(hasher)
}

/// `sha256:<hex>` of raw bytes — birth-record spans pin the bytes that were
/// actually born, uncanonicalized.
pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_digest(hasher)
}

fn hex_digest(hasher: Sha256) -> String {
    let mut out = String::with_capacity(7 + 64);
    out.push_str("sha256:");
    for b in hasher.finalize() {
        let _ = write!(out, "{b:02x}");
    }
    out
}

// ---- tokenizers ------------------------------------------------------------

/// Generic canonical tokens: maximal non-whitespace runs. No comment
/// knowledge — for unknown syntaxes, whitespace is the only churn that is
/// certainly formatting.
fn text_tokens(text: &str) -> Vec<Token> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        out.extend(line.split_whitespace().map(|w| Token {
            text: w.to_string(),
            line: i + 1,
        }));
    }
    out
}

/// Rust canonical tokens: comments stripped, string/char literals kept
/// whole, identifiers and numbers as runs, punctuation single-char.
fn rust_tokens(text: &str) -> Vec<Token> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    let mut line = 1;
    let is_ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    while i < b.len() {
        let c = b[i];
        if c == b'\n' {
            line += 1;
            i += 1;
        } else if c.is_ascii_whitespace() {
            i += 1;
        } else if c == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
        } else if c == b'/' && b.get(i + 1) == Some(&b'*') {
            let mut depth = 1;
            i += 2;
            while i < b.len() && depth > 0 {
                if b[i] == b'\n' {
                    line += 1;
                }
                if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
                    depth += 1;
                    i += 2;
                } else if b[i] == b'*' && b.get(i + 1) == Some(&b'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
        } else if c == b'"' || (c == b'r' && raw_string_start(b, i).is_some()) {
            let start = i;
            let start_line = line;
            if c == b'"' {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\n' {
                        line += 1;
                    }
                    i += if b[i] == b'\\' { 2 } else { 1 };
                }
                i = (i + 1).min(b.len());
            } else {
                let hashes = raw_string_start(b, i).expect("checked");
                i += 1 + hashes + 1; // r, #s, opening quote
                let close: Vec<u8> = std::iter::once(b'"')
                    .chain(std::iter::repeat_n(b'#', hashes))
                    .collect();
                while i < b.len() && !b[i..].starts_with(&close) {
                    if b[i] == b'\n' {
                        line += 1;
                    }
                    i += 1;
                }
                i = (i + close.len()).min(b.len());
            }
            out.push(Token {
                text: String::from_utf8_lossy(&b[start..i]).into_owned(),
                line: start_line,
            });
        } else if c == b'\'' {
            // A char literal, or a lifetime (`'a` — no closing quote).
            let start = i;
            i += 1;
            if b.get(i) == Some(&b'\\') {
                i += 2;
                while i < b.len() && b[i] != b'\'' {
                    i += 1;
                }
                i = (i + 1).min(b.len());
            } else if b.get(i).copied().is_some_and(is_ident) {
                let mut j = i;
                while j < b.len() && is_ident(b[j]) {
                    j += 1;
                }
                if b.get(j) == Some(&b'\'') && j == i + 1 {
                    i = j + 1; // 'x'
                } else {
                    i = j; // 'lifetime
                }
            } else if b.get(i).is_some() {
                i += 1;
                if b.get(i) == Some(&b'\'') {
                    i += 1;
                }
            }
            out.push(Token {
                text: String::from_utf8_lossy(&b[start..i]).into_owned(),
                line,
            });
        } else if is_ident(c) {
            let start = i;
            while i < b.len() && is_ident(b[i]) {
                i += 1;
            }
            out.push(Token {
                text: String::from_utf8_lossy(&b[start..i]).into_owned(),
                line,
            });
        } else {
            out.push(Token {
                text: (c as char).to_string(),
                line,
            });
            i += 1;
        }
    }
    out
}

/// A comma right before a closing bracket is line-wrapping, not syntax —
/// rustfmt adds and removes trailing commas as arguments wrap, and that
/// churn must not read as drift.
fn drop_trailing_commas(tokens: &mut Vec<Token>) {
    let mut kept = Vec::with_capacity(tokens.len());
    for i in 0..tokens.len() {
        let trailing = tokens[i].text == ","
            && tokens
                .get(i + 1)
                .is_some_and(|t| matches!(t.text.as_str(), ")" | "]" | "}"));
        if !trailing {
            kept.push(tokens[i].clone());
        }
    }
    *tokens = kept;
}

/// `Some(hash_count)` when `b[i..]` opens a raw string: `r"`, `r#"`, `br"`…
/// is handled by its leading `b` tokenizing as an ident — good enough for
/// hashing; only `r`-led raw strings need the lookahead.
fn raw_string_start(b: &[u8], i: usize) -> Option<usize> {
    let mut j = i + 1;
    let mut hashes = 0;
    while b.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    (b.get(j) == Some(&b'"')).then_some(hashes)
}

// ---- the item index --------------------------------------------------------

const ITEM_KEYWORDS: [&str; 11] = [
    "fn",
    "struct",
    "enum",
    "trait",
    "impl",
    "mod",
    "const",
    "static",
    "type",
    "union",
    "macro_rules",
];

/// Modifier tokens an item declaration may start with; the item's span
/// walks back over them (and over attribute groups) so `pub async fn` and
/// `#[test] fn` anchor at their first token.
const MODIFIERS: [&str; 6] = ["pub", "unsafe", "async", "extern", "default", "crate"];

fn scan_items(tokens: &[Token]) -> Vec<Item> {
    let mut items = Vec::new();
    // (scope name, index past which it closes)
    let mut scopes: Vec<(String, usize)> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        while scopes.last().is_some_and(|(_, end)| i > *end) {
            scopes.pop();
        }
        let t = tokens[i].text.as_str();
        if !ITEM_KEYWORDS.contains(&t) || !is_item_position(tokens, i) {
            i += 1;
            continue;
        }
        let Some((name, name_end)) = item_name(tokens, i) else {
            i += 1;
            continue;
        };
        let Some((sig_end, end)) = item_extent(tokens, name_end) else {
            i += 1;
            continue;
        };
        let start = walk_back_modifiers(tokens, i);
        let descend = matches!(t, "mod" | "trait") && tokens[sig_end].text == "{";
        if t == "impl" {
            // An impl block is a scope, not a linkable item — link the type
            // or a method.
            if tokens[sig_end].text == "{" {
                scopes.push((name, end));
                i = sig_end + 1;
            } else {
                i = end + 1;
            }
            continue;
        }
        let symbol = scopes
            .iter()
            .map(|(n, _)| n.as_str())
            .chain([name.as_str()])
            .collect::<Vec<_>>()
            .join("::");
        let body_tokens = &tokens[start..=end];
        let body = hash_tokens(body_tokens);
        let interface = if t == "fn" && tokens[sig_end].text == "{" {
            hash_tokens(&tokens[start..sig_end])
        } else {
            body.clone()
        };
        items.push(Item {
            symbol,
            start_line: tokens[start].line,
            end_line: tokens[end].line,
            body,
            interface,
        });
        if descend {
            scopes.push((name, end));
            i = sig_end + 1;
        } else {
            i = end + 1;
        }
    }
    items
}

/// Whether the keyword at `i` opens an item — filters `const` in `const fn`
/// (the `fn` is the item), `type`/`fn` uses inside signatures (`fn(i32)`,
/// `dyn Fn`), and keyword-looking idents after `.`/`::`.
fn is_item_position(tokens: &[Token], i: usize) -> bool {
    if i > 0 {
        let prev = tokens[i - 1].text.as_str();
        if prev == "." || prev == ":" || prev == "<" || prev == "&" || prev == "(" || prev == ","
        {
            return false;
        }
    }
    let next = tokens.get(i + 1).map(|t| t.text.as_str());
    match tokens[i].text.as_str() {
        // `const fn f` / `const unsafe fn` — the fn is the item; `const {`
        // blocks and `const _` still index (the `_` names a check).
        "const" => !matches!(next, Some("fn" | "unsafe" | "async" | "extern" | "{")),
        "static" => next != Some("fn"),
        // `union` is contextual: an item only as `union Name {`.
        "union" => {
            next.is_some_and(is_ident_token) && tokens.get(i + 2).map(|t| t.text.as_str()) == Some("{")
        }
        "macro_rules" => next == Some("!"),
        // `fn` as a type (`fn(i32) -> i32`) has `(` right after.
        "fn" => next != Some("("),
        _ => true,
    }
}

fn is_ident_token(t: &str) -> bool {
    t.bytes()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
}

/// The item's name and the index scanning continues from. For `impl`, the
/// name is the target type (`impl Trait for Type` → `Type`).
fn item_name(tokens: &[Token], kw: usize) -> Option<(String, usize)> {
    match tokens[kw].text.as_str() {
        "impl" => {
            let mut angle = 0i32;
            let mut name = None;
            let mut j = kw + 1;
            while j < tokens.len() {
                let t = tokens[j].text.as_str();
                match t {
                    "{" | ";" if angle == 0 => break,
                    "-" if tokens.get(j + 1).map(|t| t.text.as_str()) == Some(">") => j += 1,
                    "<" => angle += 1,
                    ">" => angle = (angle - 1).max(0),
                    "for" if angle == 0 => name = None,
                    "where" if angle == 0 => break,
                    _ if angle == 0 && is_ident_token(t) && t != "dyn" => {
                        name = Some(t.to_string());
                    }
                    _ => {}
                }
                j += 1;
            }
            name.map(|n| (n, j))
        }
        "macro_rules" => {
            let name = tokens.get(kw + 2)?;
            is_ident_token(&name.text).then(|| (name.text.clone(), kw + 3))
        }
        _ => {
            let name = tokens.get(kw + 1)?;
            (is_ident_token(&name.text) || name.text == "_")
                .then(|| (name.text.clone(), kw + 2))
        }
    }
}

/// From past the name, the indices of the body-opening `{` (or the
/// terminating `;`) and of the item's last token. `;` and `{` count only at
/// bracket depth zero, so `[u8; 4]` and initializer blocks don't terminate
/// early.
fn item_extent(tokens: &[Token], from: usize) -> Option<(usize, usize)> {
    let mut depth = 0i32;
    let mut j = from;
    while j < tokens.len() {
        match tokens[j].text.as_str() {
            "(" | "[" => depth += 1,
            ")" | "]" => depth -= 1,
            ";" if depth == 0 => return Some((j, j)),
            "{" if depth == 0 => {
                let mut braces = 1;
                let open = j;
                let mut k = j + 1;
                while k < tokens.len() && braces > 0 {
                    match tokens[k].text.as_str() {
                        "{" => braces += 1,
                        "}" => braces -= 1,
                        _ => {}
                    }
                    k += 1;
                }
                return (braces == 0).then_some((open, k - 1));
            }
            _ => {}
        }
        j += 1;
    }
    None
}

/// Walk back over modifiers and attribute groups so the item starts at its
/// first own token: `#[test] pub(crate) async fn f` anchors at `#`.
fn walk_back_modifiers(tokens: &[Token], kw: usize) -> usize {
    let mut start = kw;
    // `const fn` / `const unsafe fn`: the const is a modifier of the fn.
    loop {
        let Some(prev) = start.checked_sub(1) else {
            break;
        };
        let t = tokens[prev].text.as_str();
        if MODIFIERS.contains(&t) || (t == "const" && tokens[kw].text == "fn") {
            start = prev;
        } else if t == ")" {
            // pub(crate) / pub(in path)
            let mut depth = 1;
            let mut j = prev;
            while depth > 0 && j > 0 {
                j -= 1;
                match tokens[j].text.as_str() {
                    ")" => depth += 1,
                    "(" => depth -= 1,
                    _ => {}
                }
            }
            if j > 0 && tokens[j - 1].text == "pub" {
                start = j - 1;
            } else {
                break;
            }
        } else if t == "]" {
            // #[attribute(...)]
            let mut depth = 1;
            let mut j = prev;
            while depth > 0 && j > 0 {
                j -= 1;
                match tokens[j].text.as_str() {
                    "]" => depth += 1,
                    "[" => depth -= 1,
                    _ => {}
                }
            }
            if j > 0 && tokens[j - 1].text == "#" {
                start = j - 1;
            } else {
                break;
            }
        } else if t.starts_with('"') && start > 1 && tokens[start - 2].text == "extern" {
            start -= 2; // extern "C" fn
        } else {
            break;
        }
    }
    start
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canon(text: &str) -> Canonical {
        canonicalize("lib.rs", text)
    }

    fn item<'a>(c: &'a Canonical, symbol: &str) -> &'a Item {
        let found = c.find(symbol);
        assert_eq!(found.len(), 1, "`{symbol}`: {:?}", symbols(c));
        found[0]
    }

    fn symbols(c: &Canonical) -> Vec<&str> {
        c.items.iter().map(|i| i.symbol.as_str()).collect()
    }

    #[test]
    fn formatting_and_comments_never_move_hashes() {
        let a = canon("pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n");
        let b = canon(
            "// summing, now with commentary\npub fn add(\n    a: i32,\n    b: i32,\n) -> i32 {\n    /* body */ a + b\n}\n",
        );
        assert_eq!(a.file_hash(), b.file_hash());
        assert_eq!(item(&a, "add").body, item(&b, "add").body);
        assert_eq!(item(&a, "add").interface, item(&b, "add").interface);
    }

    #[test]
    fn a_fn_splits_interface_from_body() {
        let base = canon("fn f(x: u32) -> u32 { x }\n");
        let body_edit = canon("fn f(x: u32) -> u32 { x + 1 }\n");
        let sig_edit = canon("fn f(x: u64) -> u64 { x }\n");
        let (b0, b1, b2) = (item(&base, "f"), item(&body_edit, "f"), item(&sig_edit, "f"));
        assert_ne!(b0.body, b1.body);
        assert_eq!(b0.interface, b1.interface, "a body edit holds the interface");
        assert_ne!(b0.interface, b2.interface, "a signature edit moves it");
    }

    #[test]
    fn scopes_nest_through_mod_impl_and_trait() {
        let c = canon(
            "mod outer {\n\
             \x20   pub struct Wrap<T> { inner: T }\n\
             \x20   impl<T: Clone> Wrap<T> {\n\
             \x20       pub fn get(&self) -> &T { &self.inner }\n\
             \x20   }\n\
             \x20   impl std::fmt::Display for Wrap<u32> {\n\
             \x20       fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, \"{}\", self.inner) }\n\
             \x20   }\n\
             \x20   pub trait Sized2 { fn size(&self) -> usize; }\n\
             }\n\
             const LIMIT: [u8; 4] = [0; 4];\n\
             macro_rules! m { () => {} }\n",
        );
        for s in [
            "outer",
            "outer::Wrap",
            "outer::Wrap::get",
            "outer::Wrap::fmt",
            "outer::Sized2",
            "outer::Sized2::size",
            "LIMIT",
            "m",
        ] {
            assert_eq!(c.find(s).len(), 1, "`{s}` in {:?}", symbols(&c));
        }
        // Leaf-name lookup works while unique.
        assert_eq!(c.find("get")[0].symbol, "outer::Wrap::get");
    }

    #[test]
    fn attributes_and_modifiers_join_their_item() {
        let text = "struct A;\n#[derive(Debug)]\n#[repr(C)]\npub(crate) struct B { x: u8 }\n";
        let c = canon(text);
        assert_eq!(item(&c, "B").start_line, 2, "the span starts at #[derive]");
        assert_eq!(item(&c, "A").end_line, 1);
    }

    #[test]
    fn strings_comments_and_lifetimes_lex_whole() {
        let c = canon(
            "fn f<'a>(s: &'a str) -> String { format!(\"x {{}} // not a comment\", s) }\nconst R: &str = r#\"raw \" body\"#;\n",
        );
        assert_eq!(c.find("f").len(), 1);
        assert_eq!(c.find("R").len(), 1);
        // 'a lexes as a lifetime, not an unterminated char swallowing code.
        assert!(c.tokens.iter().any(|t| t.text == "'a"));
    }

    #[test]
    fn const_fn_is_one_item_and_type_uses_are_not_items() {
        let c = canon(
            "pub const fn zero() -> usize { 0 }\nfn take(f: fn(i32) -> i32, s: &dyn Fn()) { f(1); s(); }\n",
        );
        assert_eq!(symbols(&c), vec!["zero", "take"]);
    }

    #[test]
    fn generic_text_files_hash_by_whitespace_runs() {
        let a = canonicalize("schema.sql", "SELECT *\n  FROM t;\n");
        let b = canonicalize("schema.sql", "SELECT * FROM t;\n");
        assert_eq!(a.canonicalizer, TEXT_CANON);
        assert_eq!(a.file_hash(), b.file_hash());
        assert!(a.items.is_empty());
    }
}
