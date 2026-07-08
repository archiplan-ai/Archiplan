//! The `.arch` lexer: line-oriented tokens plus offside-rule indentation.
//!
//! Blank and comment-only lines are invisible to the token stream. A
//! significant line yields its tokens followed by `Newline`; indentation
//! changes yield `Indent`/`Dedent` pairs measured in leading spaces (tabs in
//! the indent are rejected). The stream ends with pending `Dedent`s and one
//! `Eof`. Comments are captured beside the tokens — with their line and
//! whether they trail code — so the attach pass can turn the ones in
//! definition position into element definitions
//! (`archi/requirements/element-definitions/`).

use super::span::{Diagnostic, FileId, Span};

/// One comment, captured for the definition attach pass. The token stream
/// never contains it.
#[derive(Clone, Debug)]
pub(crate) struct Comment {
    /// 1-based line the comment sits on.
    pub line: usize,
    /// Text after the `//`, ends trimmed.
    pub text: String,
    /// The comment including its `//`.
    pub span: Span,
    /// Whether code precedes it on its line (a comment-only line is not
    /// trailing).
    pub trailing: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Tok {
    Ident(String),
    // Reserved words.
    Import,
    Def,
    Open,
    Node,
    View,
    Rel,
    Conn,
    Port,
    Trans,
    In,
    // Punctuation.
    Dot,
    Comma,
    Star,
    LParen,
    RParen,
    Eq,
    Colon,
    ColonEq,
    /// `->`
    Arrow,
    /// `<-`
    LArrow,
    /// `<->`
    BiArrow,
    // Structure.
    Newline,
    Indent,
    Dedent,
    Eof,
}

impl Tok {
    /// How the token reads in a message.
    pub(crate) fn describe(&self) -> String {
        match self {
            Tok::Ident(s) => format!("`{s}`"),
            Tok::Import => "`import`".into(),
            Tok::Def => "`def`".into(),
            Tok::Open => "`open`".into(),
            Tok::Node => "`node`".into(),
            Tok::View => "`view`".into(),
            Tok::Rel => "`rel`".into(),
            Tok::Conn => "`conn`".into(),
            Tok::Port => "`port`".into(),
            Tok::Trans => "`trans`".into(),
            Tok::In => "`in`".into(),
            Tok::Dot => "`.`".into(),
            Tok::Comma => "`,`".into(),
            Tok::Star => "`*`".into(),
            Tok::LParen => "`(`".into(),
            Tok::RParen => "`)`".into(),
            Tok::Eq => "`=`".into(),
            Tok::Colon => "`:`".into(),
            Tok::ColonEq => "`:=`".into(),
            Tok::Arrow => "`->`".into(),
            Tok::LArrow => "`<-`".into(),
            Tok::BiArrow => "`<->`".into(),
            Tok::Newline => "end of line".into(),
            Tok::Indent => "indentation".into(),
            Tok::Dedent => "end of block".into(),
            Tok::Eof => "end of file".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Token {
    pub tok: Tok,
    pub span: Span,
}

fn keyword(word: &str) -> Option<Tok> {
    Some(match word {
        "import" => Tok::Import,
        "def" => Tok::Def,
        "open" => Tok::Open,
        "node" => Tok::Node,
        "view" => Tok::View,
        "rel" => Tok::Rel,
        "conn" => Tok::Conn,
        "port" => Tok::Port,
        "trans" => Tok::Trans,
        "in" => Tok::In,
        _ => return None,
    })
}

fn err(file: FileId, start: usize, end: usize, msg: impl Into<String>) -> Diagnostic {
    Diagnostic::new("E_PARSE", msg, Span::new(file, start, end))
}

/// Lex one file into a token stream plus its captured comments.
pub(crate) fn lex(file: FileId, src: &str) -> Result<(Vec<Token>, Vec<Comment>), Diagnostic> {
    let bytes = src.as_bytes();
    let mut tokens = Vec::new();
    let mut comments = Vec::new();
    let mut indents: Vec<usize> = vec![0];
    let mut pos = 0usize;
    let mut line_no = 0usize;

    while pos < bytes.len() {
        // --- leading indentation -----------------------------------------
        line_no += 1;
        let line_start = pos;
        let mut indent = 0usize;
        while pos < bytes.len() {
            match bytes[pos] {
                b' ' => {
                    indent += 1;
                    pos += 1;
                }
                b'\t' => {
                    return Err(err(
                        file,
                        pos,
                        pos + 1,
                        "tabs are not allowed in indentation; use spaces",
                    ));
                }
                _ => break,
            }
        }
        // Blank or comment-only lines are invisible to the offside rule.
        let rest = &src[pos..];
        let line_end_rel = rest.find('\n').map_or(rest.len(), |i| i);
        let line_body = rest[..line_end_rel].trim_end_matches('\r');
        if line_body.is_empty() || line_body.starts_with("//") {
            if let Some(text) = line_body.strip_prefix("//") {
                comments.push(Comment {
                    line: line_no,
                    text: text.trim().to_string(),
                    span: Span::new(file, pos, pos + line_body.len()),
                    trailing: false,
                });
            }
            pos += line_end_rel + usize::from(line_end_rel < rest.len());
            continue;
        }
        // --- offside rule --------------------------------------------------
        let current = *indents.last().expect("indent stack is never empty");
        if indent > current {
            indents.push(indent);
            tokens.push(Token {
                tok: Tok::Indent,
                span: Span::new(file, line_start, pos),
            });
        } else if indent < current {
            while *indents.last().expect("indent stack is never empty") > indent {
                indents.pop();
                tokens.push(Token {
                    tok: Tok::Dedent,
                    span: Span::new(file, line_start, pos),
                });
            }
            if *indents.last().expect("indent stack is never empty") != indent {
                return Err(err(
                    file,
                    line_start,
                    pos,
                    "unindent does not match any enclosing indentation level",
                ));
            }
        }
        // --- tokens of the line ---------------------------------------------
        let line_end = pos + line_body.len();
        while pos < line_end {
            let b = bytes[pos];
            match b {
                b' ' | b'\r' => {
                    pos += 1;
                }
                b'/' => {
                    if bytes.get(pos + 1) == Some(&b'/') {
                        comments.push(Comment {
                            line: line_no,
                            text: src[pos + 2..line_end].trim().to_string(),
                            span: Span::new(file, pos, line_end),
                            trailing: true,
                        });
                        pos = line_end;
                    } else {
                        return Err(err(file, pos, pos + 1, "unexpected character `/`"));
                    }
                }
                b'.' | b',' | b'*' | b'(' | b')' | b'=' => {
                    let tok = match b {
                        b'.' => Tok::Dot,
                        b',' => Tok::Comma,
                        b'*' => Tok::Star,
                        b'(' => Tok::LParen,
                        b')' => Tok::RParen,
                        _ => Tok::Eq,
                    };
                    tokens.push(Token {
                        tok,
                        span: Span::new(file, pos, pos + 1),
                    });
                    pos += 1;
                }
                b':' => {
                    if bytes.get(pos + 1) == Some(&b'=') {
                        tokens.push(Token {
                            tok: Tok::ColonEq,
                            span: Span::new(file, pos, pos + 2),
                        });
                        pos += 2;
                    } else {
                        tokens.push(Token {
                            tok: Tok::Colon,
                            span: Span::new(file, pos, pos + 1),
                        });
                        pos += 1;
                    }
                }
                b'-' => {
                    if bytes.get(pos + 1) == Some(&b'>') {
                        tokens.push(Token {
                            tok: Tok::Arrow,
                            span: Span::new(file, pos, pos + 2),
                        });
                        pos += 2;
                    } else {
                        return Err(err(file, pos, pos + 1, "expected `->`"));
                    }
                }
                b'<' => {
                    if bytes.get(pos + 1) == Some(&b'-') {
                        if bytes.get(pos + 2) == Some(&b'>') {
                            tokens.push(Token {
                                tok: Tok::BiArrow,
                                span: Span::new(file, pos, pos + 3),
                            });
                            pos += 3;
                        } else {
                            tokens.push(Token {
                                tok: Tok::LArrow,
                                span: Span::new(file, pos, pos + 2),
                            });
                            pos += 2;
                        }
                    } else {
                        return Err(err(file, pos, pos + 1, "expected `<-` or `<->`"));
                    }
                }
                c if c.is_ascii_alphabetic() || c == b'_' => {
                    let start = pos;
                    while pos < line_end
                        && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_')
                    {
                        pos += 1;
                    }
                    let word = &src[start..pos];
                    let tok = keyword(word).unwrap_or_else(|| Tok::Ident(word.to_string()));
                    tokens.push(Token {
                        tok,
                        span: Span::new(file, start, pos),
                    });
                }
                c => {
                    return Err(err(
                        file,
                        pos,
                        pos + 1,
                        format!("unexpected character `{}`", c as char),
                    ));
                }
            }
        }
        tokens.push(Token {
            tok: Tok::Newline,
            span: Span::new(file, line_end, line_end + 1),
        });
        pos = line_end + usize::from(line_end < bytes.len());
    }

    let eof = src.len();
    while indents.len() > 1 {
        indents.pop();
        tokens.push(Token {
            tok: Tok::Dedent,
            span: Span::new(file, eof, eof),
        });
    }
    tokens.push(Token {
        tok: Tok::Eof,
        span: Span::new(file, eof, eof),
    });
    Ok((tokens, comments))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::span::SourceMap;

    fn toks(src: &str) -> Vec<Tok> {
        let mut map = SourceMap::new();
        let f = map.add_file("test.arch", src);
        lex(f, src)
            .expect("lexes")
            .0
            .into_iter()
            .map(|t| t.tok)
            .collect()
    }

    fn comments(src: &str) -> Vec<Comment> {
        let mut map = SourceMap::new();
        let f = map.add_file("test.arch", src);
        lex(f, src).expect("lexes").1
    }

    fn lex_err(src: &str) -> Diagnostic {
        let mut map = SourceMap::new();
        let f = map.add_file("test.arch", src);
        lex(f, src).expect_err("must fail")
    }

    #[test]
    fn simple_line() {
        assert_eq!(
            toks("def node AuthService"),
            vec![
                Tok::Def,
                Tok::Node,
                Tok::Ident("AuthService".into()),
                Tok::Newline,
                Tok::Eof
            ]
        );
    }

    #[test]
    fn blocks_produce_indent_dedent() {
        let src = "def node A:\n  port x\n  port y\ndef node B\n";
        assert_eq!(
            toks(src),
            vec![
                Tok::Def,
                Tok::Node,
                Tok::Ident("A".into()),
                Tok::Colon,
                Tok::Newline,
                Tok::Indent,
                Tok::Port,
                Tok::Ident("x".into()),
                Tok::Newline,
                Tok::Port,
                Tok::Ident("y".into()),
                Tok::Newline,
                Tok::Dedent,
                Tok::Def,
                Tok::Node,
                Tok::Ident("B".into()),
                Tok::Newline,
                Tok::Eof
            ]
        );
    }

    #[test]
    fn blank_and_comment_lines_are_invisible() {
        let src = "def node A:\n\n  // interface\n  port x\n\n  port y\n";
        let t = toks(src);
        assert_eq!(t.iter().filter(|t| **t == Tok::Indent).count(), 1);
        assert_eq!(t.iter().filter(|t| **t == Tok::Dedent).count(), 1);
        assert_eq!(t.iter().filter(|t| **t == Tok::Port).count(), 2);
    }

    #[test]
    fn nested_dedents_unwind_fully() {
        let src = "open A:\n  def node B:\n    port x\nopen C:\n  port y\n";
        let t = toks(src);
        let indents = t.iter().filter(|t| **t == Tok::Indent).count();
        let dedents = t.iter().filter(|t| **t == Tok::Dedent).count();
        assert_eq!((indents, dedents), (3, 3));
    }

    #[test]
    fn eof_closes_open_blocks() {
        let t = toks("def node A:\n  port x");
        assert_eq!(&t[t.len() - 3..], &[Tok::Newline, Tok::Dedent, Tok::Eof]);
    }

    #[test]
    fn arrows_and_lanes() {
        assert_eq!(
            toks("def conn login := * ->LoginForm, <-AuthResponse *"),
            vec![
                Tok::Def,
                Tok::Conn,
                Tok::Ident("login".into()),
                Tok::ColonEq,
                Tok::Star,
                Tok::Arrow,
                Tok::Ident("LoginForm".into()),
                Tok::Comma,
                Tok::LArrow,
                Tok::Ident("AuthResponse".into()),
                Tok::Star,
                Tok::Newline,
                Tok::Eof
            ]
        );
        assert!(toks("a <-> b").contains(&Tok::BiArrow));
        assert!(toks("a := b").contains(&Tok::ColonEq));
    }

    #[test]
    fn comments_run_to_end_of_line() {
        assert_eq!(
            toks("port x // the login port\nport y"),
            vec![
                Tok::Port,
                Tok::Ident("x".into()),
                Tok::Newline,
                Tok::Port,
                Tok::Ident("y".into()),
                Tok::Newline,
                Tok::Eof
            ]
        );
    }

    #[test]
    fn comments_are_captured_with_lines_and_positions() {
        let src = "// header prose\n\ndef node A: // the a node\n  // above x\n  port x\n";
        let cs = comments(src);
        let view: Vec<(usize, &str, bool)> = cs
            .iter()
            .map(|c| (c.line, c.text.as_str(), c.trailing))
            .collect();
        assert_eq!(
            view,
            vec![
                (1, "header prose", false),
                (3, "the a node", true),
                (4, "above x", false),
            ]
        );
    }

    #[test]
    fn empty_comments_capture_empty_text() {
        let cs = comments("//\nport x //   \n");
        assert_eq!(cs.len(), 2);
        assert!(cs.iter().all(|c| c.text.is_empty()));
    }

    #[test]
    fn crlf_is_tolerated() {
        assert_eq!(
            toks("def node A\r\ndef node B\r\n"),
            toks("def node A\ndef node B\n")
        );
    }

    #[test]
    fn tabs_in_indentation_are_rejected() {
        let d = lex_err("def node A:\n\tport x\n");
        assert!(d.message.contains("tab"));
    }

    #[test]
    fn stray_unindent_is_rejected() {
        let d = lex_err("def node A:\n    port x\n  port y\n");
        assert!(d.message.contains("unindent"));
    }

    #[test]
    fn lone_dash_is_rejected() {
        assert!(lex_err("a - b").message.contains("->"));
    }

    #[test]
    fn spans_point_into_the_file() {
        let src = "def node Abc\n";
        let mut map = SourceMap::new();
        let f = map.add_file("m.arch", src);
        let (tokens, _) = lex(f, src).unwrap();
        let ident = tokens
            .iter()
            .find(|t| matches!(t.tok, Tok::Ident(_)))
            .unwrap();
        let (name, line, col) = map.location(ident.span);
        assert_eq!((name, line, col), ("m.arch", 1, 10));
    }
}
