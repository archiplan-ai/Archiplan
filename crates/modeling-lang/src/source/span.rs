//! Source positions and diagnostics for the `.arch` surface language.
//!
//! Every token and AST node carries a [`Span`] (byte range in one file); a
//! [`SourceMap`] turns spans back into `file:line:col` for human output. A
//! [`Diagnostic`] is a compile-time error — a parse, resolution or lowering
//! failure, or an engine error localized to the statement's origin.

use std::fmt;

/// Index of a file in a [`SourceMap`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct FileId(pub(crate) u32);

/// A byte range within one source file.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {
    /// The file the range lives in.
    pub file: FileId,
    /// Start byte offset, inclusive.
    pub start: u32,
    /// End byte offset, exclusive.
    pub end: u32,
}

impl Span {
    pub(crate) fn new(file: FileId, start: usize, end: usize) -> Self {
        Span {
            file,
            start: start as u32,
            end: end as u32,
        }
    }

    /// The span from the start of `self` to the end of `other`.
    pub(crate) fn to(self, other: Span) -> Span {
        Span {
            file: self.file,
            start: self.start,
            end: other.end,
        }
    }
}

/// A value with the span it was written at.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub(crate) fn new(value: T, span: Span) -> Self {
        Spanned { value, span }
    }
}

struct SourceFile {
    /// Display name: the path relative to the project root.
    name: String,
    /// Byte offset of each line start, for offset → line:col.
    line_starts: Vec<u32>,
}

/// The set of source files of one compilation, addressed by [`FileId`].
#[derive(Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    /// An empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a file and return its id.
    pub fn add_file(&mut self, name: impl Into<String>, text: impl Into<String>) -> FileId {
        let text = text.into();
        let mut line_starts = vec![0u32];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i as u32 + 1);
            }
        }
        self.files.push(SourceFile {
            name: name.into(),
            line_starts,
        });
        FileId(self.files.len() as u32 - 1)
    }

    /// `(file name, 1-based line, 1-based column)` of the span's start.
    pub fn location(&self, span: Span) -> (&str, u32, u32) {
        let f = &self.files[span.file.0 as usize];
        let line = f
            .line_starts
            .partition_point(|&s| s <= span.start)
            .saturating_sub(1);
        let col = span.start - f.line_starts[line];
        (&f.name, line as u32 + 1, col + 1)
    }
}

/// A compile-time failure with a source location.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    /// Stable error code (`E_PARSE`, `E_NOT_VISIBLE`, or any engine code).
    pub code: String,
    /// Human-readable one-liner.
    pub message: String,
    /// Where it happened; `None` for project-level failures (manifest, io).
    pub span: Option<Span>,
    /// Secondary locations ("first defined here", ...).
    pub notes: Vec<(String, Option<Span>)>,
}

impl Diagnostic {
    pub(crate) fn new(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Diagnostic {
            code: code.into(),
            message: message.into(),
            span: Some(span),
            notes: Vec::new(),
        }
    }

    pub(crate) fn project(code: impl Into<String>, message: impl Into<String>) -> Self {
        Diagnostic {
            code: code.into(),
            message: message.into(),
            span: None,
            notes: Vec::new(),
        }
    }

    pub(crate) fn with_note(mut self, message: impl Into<String>, span: Option<Span>) -> Self {
        self.notes.push((message.into(), span));
        self
    }

    /// Render as `file:line:col: CODE: message`, one line per note.
    pub fn render(&self, map: &SourceMap) -> String {
        let mut out = String::new();
        match self.span {
            Some(s) => {
                let (name, line, col) = map.location(s);
                out.push_str(&format!(
                    "{name}:{line}:{col}: {}: {}",
                    self.code, self.message
                ));
            }
            None => out.push_str(&format!("{}: {}", self.code, self.message)),
        }
        for (msg, span) in &self.notes {
            out.push('\n');
            match span {
                Some(s) => {
                    let (name, line, col) = map.location(*s);
                    out.push_str(&format!("  {name}:{line}:{col}: {msg}"));
                }
                None => out.push_str(&format!("  {msg}")),
            }
        }
        out
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
