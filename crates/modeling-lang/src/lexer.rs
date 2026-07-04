//! Tokenizer for the statement language.

use crate::error::{ErrorCode, LangError};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Tok {
    Ident,
    KwNode,
    KwView,
    KwRel,
    KwConn,
    KwTrans,
    KwOpen,
    KwRename,
    KwDelete,
    KwUntag,
    KwIn,
    KwPorts,
    KwCheck,
    KwDump,
    Assign,  // :=
    Arrow,   // ->
    BiArrow, // <->
    Eq,      // =
    Star,
    Dot,
    Comma,
    Semi,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Eof,
}

impl Tok {
    pub(crate) fn describe(self) -> &'static str {
        match self {
            Tok::Ident => "a name",
            Tok::KwNode => "`node`",
            Tok::KwView => "`view`",
            Tok::KwRel => "`rel`",
            Tok::KwConn => "`conn`",
            Tok::KwTrans => "`trans`",
            Tok::KwOpen => "`open`",
            Tok::KwRename => "`rename`",
            Tok::KwDelete => "`delete`",
            Tok::KwUntag => "`untag`",
            Tok::KwIn => "`in`",
            Tok::KwPorts => "`ports`",
            Tok::KwCheck => "`check`",
            Tok::KwDump => "`dump`",
            Tok::Assign => "`:=`",
            Tok::Arrow => "`->`",
            Tok::BiArrow => "`<->`",
            Tok::Eq => "`=`",
            Tok::Star => "`*`",
            Tok::Dot => "`.`",
            Tok::Comma => "`,`",
            Tok::Semi => "`;`",
            Tok::LParen => "`(`",
            Tok::RParen => "`)`",
            Tok::LBrace => "`{`",
            Tok::RBrace => "`}`",
            Tok::Eof => "end of input",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Token {
    pub tok: Tok,
    pub start: usize,
    pub end: usize,
}

pub(crate) fn line_col(src: &str, pos: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in src.char_indices() {
        if i >= pos {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

pub(crate) fn line_at(src: &str, pos: usize) -> &str {
    let start = src[..pos.min(src.len())].rfind('\n').map_or(0, |i| i + 1);
    let end = src[start..].find('\n').map_or(src.len(), |i| start + i);
    src[start..end].trim()
}

fn parse_err(src: &str, pos: usize, message: String) -> LangError {
    let (line, col) = line_col(src, pos);
    let mut e = LangError::new(ErrorCode::Parse, format!("{message} at {line}:{col}"));
    e.subject = line_at(src, pos).to_string();
    e
}

fn keyword(word: &str) -> Option<Tok> {
    Some(match word {
        "node" => Tok::KwNode,
        "view" => Tok::KwView,
        "rel" => Tok::KwRel,
        "conn" => Tok::KwConn,
        "trans" => Tok::KwTrans,
        "open" => Tok::KwOpen,
        "rename" => Tok::KwRename,
        "delete" => Tok::KwDelete,
        "untag" => Tok::KwUntag,
        "in" => Tok::KwIn,
        "ports" => Tok::KwPorts,
        "check" => Tok::KwCheck,
        "dump" => Tok::KwDump,
        _ => return None,
    })
}

pub(crate) fn lex(src: &str) -> Result<Vec<Token>, LangError> {
    let bytes = src.as_bytes();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        match c {
            ' ' | '\t' | '\r' | '\n' => {
                i += 1;
            }
            '#' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let word = &src[start..i];
                let tok = keyword(word).unwrap_or(Tok::Ident);
                toks.push(Token { tok, start, end: i });
            }
            ':' => {
                if bytes.get(i + 1) == Some(&b'=') {
                    toks.push(Token {
                        tok: Tok::Assign,
                        start: i,
                        end: i + 2,
                    });
                    i += 2;
                } else {
                    return Err(parse_err(src, i, "expected `:=`".to_string()));
                }
            }
            '-' => {
                if bytes.get(i + 1) == Some(&b'>') {
                    toks.push(Token {
                        tok: Tok::Arrow,
                        start: i,
                        end: i + 2,
                    });
                    i += 2;
                } else {
                    return Err(parse_err(src, i, "expected `->`".to_string()));
                }
            }
            '<' => {
                if bytes.get(i + 1) == Some(&b'-') && bytes.get(i + 2) == Some(&b'>') {
                    toks.push(Token {
                        tok: Tok::BiArrow,
                        start: i,
                        end: i + 3,
                    });
                    i += 3;
                } else {
                    return Err(parse_err(src, i, "expected `<->`".to_string()));
                }
            }
            '=' => {
                toks.push(Token {
                    tok: Tok::Eq,
                    start: i,
                    end: i + 1,
                });
                i += 1;
            }
            '*' => {
                toks.push(Token {
                    tok: Tok::Star,
                    start: i,
                    end: i + 1,
                });
                i += 1;
            }
            '.' => {
                toks.push(Token {
                    tok: Tok::Dot,
                    start: i,
                    end: i + 1,
                });
                i += 1;
            }
            ',' => {
                toks.push(Token {
                    tok: Tok::Comma,
                    start: i,
                    end: i + 1,
                });
                i += 1;
            }
            ';' => {
                toks.push(Token {
                    tok: Tok::Semi,
                    start: i,
                    end: i + 1,
                });
                i += 1;
            }
            '(' => {
                toks.push(Token {
                    tok: Tok::LParen,
                    start: i,
                    end: i + 1,
                });
                i += 1;
            }
            ')' => {
                toks.push(Token {
                    tok: Tok::RParen,
                    start: i,
                    end: i + 1,
                });
                i += 1;
            }
            '{' => {
                toks.push(Token {
                    tok: Tok::LBrace,
                    start: i,
                    end: i + 1,
                });
                i += 1;
            }
            '}' => {
                toks.push(Token {
                    tok: Tok::RBrace,
                    start: i,
                    end: i + 1,
                });
                i += 1;
            }
            other => {
                return Err(parse_err(src, i, format!("unexpected character `{other}`")));
            }
        }
    }
    toks.push(Token {
        tok: Tok::Eof,
        start: src.len(),
        end: src.len(),
    });
    Ok(toks)
}
