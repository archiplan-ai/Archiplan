//! A reader for the structured markdown of doc sources
//! (`archi/requirements/spec-docs/every-field-is-present.md`): a YAML-subset frontmatter
//! block, one H1 name, and `#`-headed sections over free prose. Structure
//! only — the reader never interprets prose, and a file yields at most one
//! structural error; field- and schema-level validation happens in
//! [`super::schema`] on structurally sound documents.

/// One structural parse failure.
#[derive(Debug)]
pub struct MdError {
    /// Human-readable one-liner.
    pub message: String,
    /// 1-based line the failure is at.
    pub line: usize,
}

fn err(message: impl Into<String>, line: usize) -> MdError {
    MdError {
        message: message.into(),
        line,
    }
}

/// A frontmatter field value: a scalar (possibly empty — empty is an
/// explicit state) or an inline `[a, b]` list.
pub enum FieldValue {
    /// Everything after `key:`, trimmed.
    Scalar(String),
    /// The entries of `[a, b]`, trimmed, empties dropped.
    List(Vec<String>),
}

/// One `key: value` frontmatter field.
pub struct Field {
    /// The key, trimmed.
    pub key: String,
    /// The parsed value.
    pub value: FieldValue,
    /// 1-based line the field is on.
    pub line: usize,
}

/// A `##`-or-deeper heading and the prose under it.
pub struct Heading {
    /// Heading level: 2 for `##`, 3 for `###`, …
    pub level: usize,
    /// The heading text, trimmed.
    pub text: String,
    /// 1-based line of the heading.
    pub line: usize,
    /// Non-blank lines strictly between this heading and the next one (any
    /// level), as `(line, text)`.
    pub content: Vec<(usize, String)>,
}

/// A structurally parsed document.
pub struct MdDoc {
    /// The frontmatter fields, in file order; `None` when the file opens
    /// without a `---` block.
    pub frontmatter: Option<Vec<Field>>,
    /// The H1 text — the primitive's name.
    pub name: String,
    /// 1-based line of the H1.
    pub name_line: usize,
    /// Non-blank lines between the H1 and the first section heading — the
    /// summary-first body.
    pub summary: Vec<(usize, String)>,
    /// Every heading below H1, in file order.
    pub headings: Vec<Heading>,
}

/// Parse a document's structure. Headings inside fenced code blocks are
/// prose; a heading is a `#`+ run at column zero followed by a space.
pub fn parse(text: &str) -> Result<MdDoc, MdError> {
    let lines: Vec<&str> = text.lines().collect();
    let mut at = 0;

    let frontmatter = if lines.first().map(|l| l.trim_end()) == Some("---") {
        let mut fields: Vec<Field> = Vec::new();
        let mut close = None;
        for (i, raw) in lines.iter().enumerate().skip(1) {
            let line = i + 1;
            if raw.trim_end() == "---" {
                close = Some(i);
                break;
            }
            let Some((key, value)) = raw.split_once(':') else {
                return Err(err("frontmatter lines are `key: value` fields", line));
            };
            let key = key.trim();
            if key.is_empty() || key.contains(char::is_whitespace) {
                return Err(err("frontmatter lines are `key: value` fields", line));
            }
            if fields.iter().any(|f| f.key == key) {
                return Err(err(format!("duplicate frontmatter field `{key}`"), line));
            }
            let value = parse_value(value.trim()).map_err(|m| err(m, line))?;
            fields.push(Field {
                key: key.to_string(),
                value,
                line,
            });
        }
        let Some(close) = close else {
            return Err(err("unterminated frontmatter: no closing `---`", 1));
        };
        at = close + 1;
        Some(fields)
    } else {
        None
    };

    let mut name: Option<(String, usize)> = None;
    let mut summary: Vec<(usize, String)> = Vec::new();
    let mut headings: Vec<Heading> = Vec::new();
    let mut in_fence = false;
    for (i, raw) in lines.iter().enumerate().skip(at) {
        let line = i + 1;
        if !in_fence && raw.starts_with('#') {
            let level = raw.bytes().take_while(|b| *b == b'#').count();
            if raw.as_bytes().get(level) == Some(&b' ') {
                let text = raw[level + 1..].trim().to_string();
                match (level, &name) {
                    (1, None) => name = Some((text, line)),
                    (1, Some(_)) => {
                        return Err(err("a document has one H1: its name", line));
                    }
                    (_, None) => {
                        return Err(err("a document opens with its name as an H1", line));
                    }
                    (_, Some(_)) => headings.push(Heading {
                        level,
                        text,
                        line,
                        content: Vec::new(),
                    }),
                }
                continue;
            }
        }
        if raw.trim().is_empty() {
            continue;
        }
        if raw.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
        match (&name, headings.last_mut()) {
            (None, _) => return Err(err("a document opens with its name as an H1", line)),
            (Some(_), None) => summary.push((line, raw.to_string())),
            (Some(_), Some(h)) => h.content.push((line, raw.to_string())),
        }
    }
    let Some((name, name_line)) = name else {
        return Err(err("a document opens with its name as an H1", at + 1));
    };
    Ok(MdDoc {
        frontmatter,
        name,
        name_line,
        summary,
        headings,
    })
}

fn parse_value(v: &str) -> Result<FieldValue, String> {
    let Some(rest) = v.strip_prefix('[') else {
        return Ok(FieldValue::Scalar(v.to_string()));
    };
    let Some(inner) = rest.strip_suffix(']') else {
        return Err("unterminated list: `[a, b]`".into());
    };
    Ok(FieldValue::List(
        inner
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
    ))
}

/// The slug a name derives to: lowercased, runs of non-alphanumerics
/// collapsed to `-` (`archi/requirements/spec-docs/slugs-are-the-reference-currency.md`,
/// `archi/requirements/spec-docs/slugs-are-the-reference-currency.md`).
pub fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut gap = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            if gap && !out.is_empty() {
                out.push('-');
            }
            gap = false;
            out.push(c.to_ascii_lowercase());
        } else {
            gap = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_derive_kebab() {
        assert_eq!(
            slugify("No plaintext credentials"),
            "no-plaintext-credentials"
        );
        assert_eq!(slugify("  100× the rate!  "), "100-the-rate");
        assert_eq!(slugify("Type-Level"), "type-level");
    }

    #[test]
    fn structure_parses() {
        let doc = parse(
            "---\nkind: functional\nsatisfied-by: [A.B, C]\n---\n\n# The Name\n\nsummary\n\n## Satisfy\n\n```\n# not a heading\n```\n",
        )
        .unwrap();
        let fm = doc.frontmatter.as_ref().unwrap();
        assert_eq!(fm.len(), 2);
        assert!(matches!(&fm[1].value, FieldValue::List(v) if v == &["A.B", "C"]));
        assert_eq!((doc.name.as_str(), doc.name_line), ("The Name", 6));
        assert_eq!(doc.summary.len(), 1);
        let [satisfy] = &doc.headings[..] else {
            panic!("one heading");
        };
        // The fence swallowed the fake heading; its three lines are content.
        assert_eq!(
            (satisfy.text.as_str(), satisfy.content.len()),
            ("Satisfy", 3)
        );
    }

    #[test]
    fn structural_errors() {
        assert!(parse("prose before name\n# N\n").is_err());
        assert!(parse("# A\n# B\n").is_err());
        assert!(parse("---\nkind: x\n").is_err());
        assert!(parse("---\nkind: x\nkind: y\n---\n# N\n").is_err());
        assert!(parse("---\naffects: [a, b\n---\n# N\n").is_err());
        assert!(parse("").is_err());
    }
}
