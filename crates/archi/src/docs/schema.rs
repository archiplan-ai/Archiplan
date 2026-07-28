//! Per-primitive schemas over [`super::md`] documents: intents, requirements,
//! sessions, stressors and decisions as `archi/requirements/spec-docs/`
//! define them. Parsing is best-effort — every deviation lands in the
//! diagnostics and the parsed value keeps what was sound, with `Option`
//! marking fields the cross-checks must not trust.

use super::DocDiagnostic;
use super::md::{Field, FieldValue, Heading, MdDoc, slugify};
use crate::axes::{self, Axis};

/// Where a requirement came from (`archi/requirements/spec-docs/origin-records-why-placement-records-where.md`).
pub enum Origin {
    /// Derived directly from the enclosing intent.
    Intent,
    /// A pure refinement of the parent requirement.
    Parent,
    /// The answer to one or more breaking stressors.
    Stressors(Vec<String>),
    /// Emerged at the junction of the named requirements.
    Fusion(Vec<String>),
}

/// A stressor's outcome (`archi/requirements/spec-docs/a-stressor-presses-one-hypothesis.md`,
/// `archi/requirements/spec-docs/accept-is-a-signed-break.md`).
#[derive(Clone, Copy, PartialEq)]
pub enum Outcome {
    /// The session has not decided yet.
    Pending,
    /// The architecture holds.
    Surviving,
    /// The architecture bends; requirements are derived.
    Breaking,
    /// The architecture bends and the break is kept: a linked decision
    /// records the trade, no requirements are derived.
    Accepted,
}

/// The machine fields a file-scale requirement owns. Section-scale
/// requirements have none — they inherit
/// (`archi/requirements/spec-docs/one-claim-one-file.md`).
pub struct ReqFields {
    /// Parsed origin and its line; `None` when invalid (already reported).
    pub origin: Option<(Origin, usize)>,
    /// `satisfied-by` entries with their field line; `None` when invalid.
    pub satisfied_by: Option<(Vec<String>, usize)>,
    /// The deferral reason (empty = not deferred); `None` when invalid.
    pub deferred: Option<String>,
    /// Verification bullets of the `Satisfy` section.
    pub verifications: usize,
}

impl ReqFields {
    /// Whether this requirement claims satisfaction.
    pub fn satisfied(&self) -> bool {
        self.satisfied_by
            .as_ref()
            .is_some_and(|(v, _)| !v.is_empty())
    }

    /// Whether a deferral is in force.
    pub fn deferred(&self) -> bool {
        self.deferred.as_ref().is_some_and(|d| !d.is_empty())
    }
}

/// One requirement, at any scale.
pub struct Requirement {
    /// The slug — the filename for file scale, derived for sections.
    pub slug: String,
    /// Project-relative path of the file holding it.
    pub file: String,
    /// 1-based line of its name (H1 or section heading).
    pub line: usize,
    /// Slug of the parent requirement; `None` at an intent folder's root.
    pub parent: Option<String>,
    /// Whether it sits at the root of an intent folder.
    pub at_intent_root: bool,
    /// Own machine fields; `None` for section scale.
    pub fields: Option<ReqFields>,
}

/// An intent — the anchor of a requirements area (`archi/requirements/spec-docs/an-intent-is-a-problem-statement.md`).
pub struct Intent {
    /// The slug (= folder = filename).
    pub slug: String,
    /// Project-relative path.
    pub file: String,
    /// 1-based line of the name.
    pub line: usize,
}

/// A stress session (`archi/requirements/spec-docs/breaking-derives-requirements.md`).
pub struct Session {
    /// The slug (= folder = filename).
    pub slug: String,
    /// Project-relative path.
    pub file: String,
    /// 1-based line of the name.
    pub line: usize,
    /// The pinned version id and its line; `None` when invalid.
    pub version: Option<(String, usize)>,
    /// The closing version id (empty while open) and its line.
    pub closed: Option<(String, usize)>,
    /// The closing version's content hash, stamped by the closing verbs;
    /// absent on pre-hash records.
    pub version_hash: Option<(String, usize)>,
    /// Rounds folded into this record (`## Folded:` sections), in file order.
    pub folded: Vec<Folded>,
}

/// One folded round inside a session file: the trace `archi session fold`
/// writes and the schema validates forever.
pub struct Folded {
    /// The heading label — the folded round's slug or its merge-side label.
    pub label: String,
    /// The folded round's `closed:` trailer — empty for a round folded open,
    /// an id for one folded sealed, `pending remint` awaiting the re-mint.
    pub closed: Option<(String, usize)>,
}

impl Session {
    /// Whether the session is open (a sound, empty `closed` field).
    pub fn open(&self) -> bool {
        self.closed.as_ref().is_some_and(|(v, _)| v.is_empty())
    }
}

/// A stressor (`archi/requirements/spec-docs/a-stressor-presses-one-hypothesis.md`).
pub struct Stressor {
    /// The slug (= filename).
    pub slug: String,
    /// Project-relative path.
    pub file: String,
    /// 1-based line of the name.
    pub line: usize,
    /// Slug of the session whose folder holds it.
    pub session: String,
    /// The affects entries with the field's line; `None` when invalid.
    pub affects: Option<(Vec<String>, usize)>,
    /// The outcome; `None` when invalid.
    pub outcome: Option<Outcome>,
}

/// A decision — the priced record of one trade
/// (`archi/requirements/spec-docs/a-decision-prices-the-fork.md`). The sole
/// carrier of trade-off axes: `prefer` names what it buys, `over` what it
/// pays, both zero-or-more (empty is a valid non-comparative record).
pub struct Decision {
    /// The slug (= filename).
    pub slug: String,
    /// Project-relative path.
    pub file: String,
    /// 1-based line of the name.
    pub line: usize,
    /// What the decision is about — doc slugs and model elements — with the
    /// field's line; `None` when invalid.
    pub links: Option<(Vec<String>, usize)>,
    /// Axis labels the decision favours; `None` when invalid.
    pub prefer: Option<(Vec<String>, usize)>,
    /// Axis labels it sacrifices; `None` when invalid.
    pub over: Option<(Vec<String>, usize)>,
}

const RESERVED: [&str; 2] = ["System Context", "Satisfy"];

/// Parse an intent document.
pub fn intent(doc: &MdDoc, file: &str, stem: &str, diags: &mut Vec<DocDiagnostic>) -> Intent {
    if doc.frontmatter.is_some() {
        diags.push(DocDiagnostic::new(
            "E_DOC",
            "an intent has no frontmatter — its schema is a name and the problem statement",
            file,
            1,
        ));
    }
    if let Some(h) = doc.headings.first() {
        diags.push(DocDiagnostic::new(
            "E_DOC",
            "an intent is its name and problem statement — it holds no sections",
            file,
            h.line,
        ));
    }
    name_checks(doc, file, stem, "an intent", diags);
    Intent {
        slug: stem.to_string(),
        file: file.to_string(),
        line: doc.name_line,
    }
}

/// Parse a file-scale requirement, returning it and its section-scale
/// subrequirements.
pub fn requirement_file(
    doc: &MdDoc,
    file: &str,
    stem: &str,
    parent: Option<String>,
    at_intent_root: bool,
    diags: &mut Vec<DocDiagnostic>,
) -> (Requirement, Vec<Requirement>) {
    name_checks(doc, file, stem, "a requirement", diags);
    let fm = frontmatter(
        doc,
        file,
        &["kind", "origin", "satisfied-by", "deferred"],
        "kind, origin, satisfied-by, deferred",
        diags,
    );

    if let Some((kind, line)) = scalar(fm, "kind", file, diags)
        && !matches!(kind.as_str(), "functional" | "non-functional")
    {
        diags.push(DocDiagnostic::new(
            "E_DOC",
            format!("`kind` is `functional` or `non-functional`, got `{kind}`"),
            file,
            line,
        ));
    }
    let origin = scalar(fm, "origin", file, diags).and_then(|(v, line)| match parse_origin(&v) {
        Ok(o) => Some((o, line)),
        Err(m) => {
            diags.push(DocDiagnostic::new("E_DOC", m, file, line));
            None
        }
    });
    let satisfied_by = list(fm, "satisfied-by", file, diags);
    let deferred = scalar(fm, "deferred", file, diags);

    let mut sections = Vec::new();
    let mut verifications = 0;
    if let Some((satisfy, subs)) = requirement_sections(doc, file, diags) {
        let satisfy_content = !satisfy.content.is_empty();
        for (bullet_line, text) in &satisfy.content {
            match verification_bullet(text) {
                BulletShape::Verification => verifications += 1,
                BulletShape::TagShapedMiss => diags.push(DocDiagnostic::new(
                    "E_DOC",
                    "a verification bullet reads `- test — …` or `- type-level — …` \
                     (separator `-`, `–` or `—`); this opens with the tag but misses the shape",
                    file,
                    *bullet_line,
                )),
                BulletShape::Prose => {}
            }
        }
        if let Some((entries, line)) = &satisfied_by
            && entries.is_empty() == satisfy_content
        {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                "`satisfied-by` and the `Satisfy` prose hold together — fill both or leave both empty",
                file,
                *line,
            ));
        }
        sections = subs
            .into_iter()
            .map(|(h, sub_parent)| {
                if h.content.is_empty() {
                    diags.push(DocDiagnostic::new(
                        "E_DOC",
                        format!("subrequirement `{}` needs a summary paragraph", h.text),
                        file,
                        h.line,
                    ));
                }
                Requirement {
                    slug: slugify(&h.text),
                    file: file.to_string(),
                    line: h.line,
                    parent: Some(sub_parent.unwrap_or_else(|| stem.to_string())),
                    at_intent_root: false,
                    fields: None,
                }
            })
            .collect();
    }
    if let (Some((d, dline)), Some((entries, _))) = (&deferred, &satisfied_by)
        && !d.is_empty()
        && !entries.is_empty()
    {
        diags.push(DocDiagnostic::new(
            "E_DOC",
            "a satisfied requirement cannot be deferred — every requirement is in exactly one state",
            file,
            *dline,
        ));
    }

    let req = Requirement {
        slug: stem.to_string(),
        file: file.to_string(),
        line: doc.name_line,
        parent,
        at_intent_root,
        fields: Some(ReqFields {
            origin,
            satisfied_by,
            deferred: deferred.map(|(v, _)| v),
            verifications,
        }),
    };
    (req, sections)
}

/// Parse a session document.
pub fn session(doc: &MdDoc, file: &str, stem: &str, diags: &mut Vec<DocDiagnostic>) -> Session {
    name_checks(doc, file, stem, "a session", diags);
    let mut folded = Vec::new();
    for h in &doc.headings {
        let label = (h.level == 2)
            .then(|| h.text.strip_prefix("Folded: "))
            .flatten()
            .filter(|l| !l.trim().is_empty());
        let Some(label) = label else {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                "a session file is its name and charter — its only sections are the \
                 `## Folded: <label>` records `archi session fold` writes",
                file,
                h.line,
            ));
            continue;
        };
        folded.push(folded_section(h, label, file, diags));
    }
    let fm = frontmatter(
        doc,
        file,
        &["version", "closed", "version-hash"],
        "version, closed, version-hash",
        diags,
    );
    Session {
        slug: stem.to_string(),
        file: file.to_string(),
        line: doc.name_line,
        version: scalar(fm, "version", file, diags),
        closed: scalar(fm, "closed", file, diags),
        version_hash: optional_scalar(fm, "version-hash", file, diags),
        folded,
    }
}

/// Validate one `## Folded:` section: the folded charter's prose, then the
/// `pin:` / `closed:` / `note:` trailer, each exactly once.
fn folded_section(
    h: &Heading,
    label: &str,
    file: &str,
    diags: &mut Vec<DocDiagnostic>,
) -> Folded {
    let mut pin = None;
    let mut closed: Option<(String, usize)> = None;
    let mut note = None;
    for (line, text) in &h.content {
        match text.split_once(':').map(|(k, v)| (k, v.trim())) {
            Some(("pin", v)) if pin.is_none() => pin = Some((v.to_string(), *line)),
            Some(("closed", v)) if closed.is_none() => closed = Some((v.to_string(), *line)),
            Some(("note", v)) if note.is_none() => note = Some((v.to_string(), *line)),
            _ => {}
        }
    }
    for (key, missing) in [
        ("pin", pin.is_none()),
        ("closed", closed.is_none()),
        ("note", note.as_ref().is_none_or(|(n, _): &(String, usize)| n.is_empty())),
    ] {
        if missing {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!(
                    "a folded round records its charter and a `{key}:` trailer — \
                     `pin: <id>`, `closed: <id|pending remint|empty>`, `note: <why>`"
                ),
                file,
                h.line,
            ));
        }
    }
    Folded {
        label: label.to_string(),
        closed,
    }
}

/// Parse a stressor document.
pub fn stressor(
    doc: &MdDoc,
    file: &str,
    stem: &str,
    session: &str,
    diags: &mut Vec<DocDiagnostic>,
) -> Stressor {
    name_checks(doc, file, stem, "a stressor", diags);
    let fm = frontmatter(
        doc,
        file,
        &["affects", "outcome"],
        "affects, outcome",
        diags,
    );
    let affects = list(fm, "affects", file, diags);
    if let Some((entries, line)) = &affects
        && entries.is_empty()
    {
        diags.push(DocDiagnostic::new(
            "E_AFFECTS_EMPTY",
            "a stressor that affects nothing is not a stressor — delete the file if it is obsolete",
            file,
            *line,
        ));
    }
    let outcome = scalar(fm, "outcome", file, diags).and_then(|(v, line)| match v.as_str() {
        "pending" => Some(Outcome::Pending),
        "surviving" => Some(Outcome::Surviving),
        "breaking" => Some(Outcome::Breaking),
        "accepted" => Some(Outcome::Accepted),
        other => {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!(
                    "`outcome` is `pending`, `surviving`, `breaking` or `accepted`, got `{other}`"
                ),
                file,
                line,
            ));
            None
        }
    });

    let sections_ok = doc.headings.len() == 2
        && doc.headings[0].level == 2
        && doc.headings[0].text == "Attractor"
        && doc.headings[1].level == 2
        && doc.headings[1].text == "Resolution";
    if !sections_ok {
        let line = doc.headings.first().map_or(doc.name_line, |h| h.line);
        diags.push(DocDiagnostic::new(
            "E_DOC",
            "a stressor's sections are `Attractor` then `Resolution`, nothing else",
            file,
            line,
        ));
    } else if let Some(outcome) = outcome {
        let resolution = &doc.headings[1];
        if (outcome == Outcome::Pending) != resolution.content.is_empty() {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                "`Resolution` is non-empty exactly when the outcome is decided",
                file,
                resolution.line,
            ));
        }
    }

    Stressor {
        slug: stem.to_string(),
        file: file.to_string(),
        line: doc.name_line,
        session: session.to_string(),
        affects,
        outcome,
    }
}

/// Parse a decision document: `links`, `prefer` and `over` in frontmatter,
/// then the name and the rationale prose — a decision is atomic, it holds
/// no sections. An axis on both sides of one trade is a contradiction;
/// off-list labels are legal here and surface as findings, never errors.
pub fn decision(doc: &MdDoc, file: &str, stem: &str, diags: &mut Vec<DocDiagnostic>) -> Decision {
    name_checks(doc, file, stem, "a decision", diags);
    if let Some(h) = doc.headings.first() {
        diags.push(DocDiagnostic::new(
            "E_DOC",
            "a decision is its name, its rationale and its trade — it holds no sections",
            file,
            h.line,
        ));
    }
    let fm = frontmatter(
        doc,
        file,
        &["links", "prefer", "over"],
        "links, prefer, over",
        diags,
    );
    let links = list(fm, "links", file, diags);
    let prefer = list(fm, "prefer", file, diags);
    let over = list(fm, "over", file, diags);
    if let (Some((p, _)), Some((o, line))) = (&prefer, &over) {
        let preferred = axes::parse_list(p);
        for (raw, axis) in o.iter().zip(axes::parse_list(o)) {
            if preferred.contains(&axis) {
                diags.push(DocDiagnostic::new(
                    "E_DOC",
                    format!("`{raw}` is both preferred and sacrificed — a trade has two sides"),
                    file,
                    *line,
                ));
            }
        }
    }
    Decision {
        slug: stem.to_string(),
        file: file.to_string(),
        line: doc.name_line,
        links,
        prefer,
        over,
    }
}

/// The off-list labels of a parsed axis list: everything [`Axis::parse`]
/// keeps as `Other` — legal, recorded verbatim, surfaced as findings.
pub fn off_list(entries: &[String]) -> Vec<String> {
    axes::parse_list(entries)
        .into_iter()
        .filter_map(|a| match a {
            Axis::Other(label) => Some(label),
            _ => None,
        })
        .collect()
}

/// The name derives to the filename, and a summary paragraph follows it.
fn name_checks(doc: &MdDoc, file: &str, stem: &str, what: &str, diags: &mut Vec<DocDiagnostic>) {
    let derived = slugify(&doc.name);
    if derived != stem {
        diags.push(DocDiagnostic::new(
            "E_SLUG",
            format!(
                "`{}` derives to `{derived}`, but the file is `{stem}.md`",
                doc.name
            ),
            file,
            doc.name_line,
        ));
    }
    if doc.summary.is_empty() {
        diags.push(DocDiagnostic::new(
            "E_DOC",
            format!("{what} needs a summary paragraph after its name"),
            file,
            doc.name_line,
        ));
    }
}

/// The frontmatter block, checked for unknown fields; `None` when absent.
fn frontmatter<'a>(
    doc: &'a MdDoc,
    file: &str,
    allowed: &[&str],
    schema: &str,
    diags: &mut Vec<DocDiagnostic>,
) -> Option<&'a [Field]> {
    let Some(fields) = doc.frontmatter.as_deref() else {
        diags.push(DocDiagnostic::new(
            "E_DOC",
            format!("missing frontmatter — the machine fields are: {schema}"),
            file,
            1,
        ));
        return None;
    };
    for f in fields {
        if !allowed.contains(&f.key.as_str()) {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!(
                    "unknown frontmatter field `{}` — the schema is: {schema}",
                    f.key
                ),
                file,
                f.line,
            ));
        }
    }
    Some(fields)
}

/// A scalar field. Absence is ambiguity — a missing field is `E_DOC`, an
/// empty value is an explicit state.
fn scalar(
    fm: Option<&[Field]>,
    key: &str,
    file: &str,
    diags: &mut Vec<DocDiagnostic>,
) -> Option<(String, usize)> {
    match fm?.iter().find(|f| f.key == key) {
        None => {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!("missing frontmatter field `{key}` — empty is a state, absence is not"),
                file,
                1,
            ));
            None
        }
        Some(f) => match &f.value {
            FieldValue::Scalar(s) => Some((s.clone(), f.line)),
            FieldValue::List(_) => {
                diags.push(DocDiagnostic::new(
                    "E_DOC",
                    format!("`{key}` is a scalar field, not a list"),
                    file,
                    f.line,
                ));
                None
            }
        },
    }
}

/// A scalar field that may be absent — absence stays legal (pre-hash
/// records); a list is still a fault.
fn optional_scalar(
    fm: Option<&[Field]>,
    key: &str,
    file: &str,
    diags: &mut Vec<DocDiagnostic>,
) -> Option<(String, usize)> {
    let f = fm?.iter().find(|f| f.key == key)?;
    match &f.value {
        FieldValue::Scalar(s) => Some((s.clone(), f.line)),
        FieldValue::List(_) => {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!("`{key}` is a scalar field, not a list"),
                file,
                f.line,
            ));
            None
        }
    }
}

/// A list field; an empty scalar reads as the explicit empty list.
fn list(
    fm: Option<&[Field]>,
    key: &str,
    file: &str,
    diags: &mut Vec<DocDiagnostic>,
) -> Option<(Vec<String>, usize)> {
    match fm?.iter().find(|f| f.key == key) {
        None => {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!("missing frontmatter field `{key}` — empty is a state, absence is not"),
                file,
                1,
            ));
            None
        }
        Some(f) => match &f.value {
            FieldValue::List(v) => Some((v.clone(), f.line)),
            FieldValue::Scalar(s) if s.is_empty() => Some((Vec::new(), f.line)),
            FieldValue::Scalar(_) => {
                diags.push(DocDiagnostic::new(
                    "E_DOC",
                    format!("`{key}` is a list field: `[a, b]`"),
                    file,
                    f.line,
                ));
                None
            }
        },
    }
}

pub(crate) fn parse_origin(s: &str) -> Result<Origin, String> {
    const FORMS: &str = "origin is `intent`, `parent`, `stressor(slug, …)` or `fusion(slug, …)`";
    let slugs = |inner: &str, form: &str| -> Result<Vec<String>, String> {
        let v: Vec<String> = inner
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        if v.is_empty() {
            Err(format!("`{form}(…)` names at least one slug"))
        } else {
            Ok(v)
        }
    };
    match s {
        "intent" => Ok(Origin::Intent),
        "parent" => Ok(Origin::Parent),
        other => {
            if let Some(inner) = other
                .strip_prefix("stressor(")
                .and_then(|r| r.strip_suffix(')'))
            {
                slugs(inner, "stressor").map(Origin::Stressors)
            } else if let Some(inner) = other
                .strip_prefix("fusion(")
                .and_then(|r| r.strip_suffix(')'))
            {
                slugs(inner, "fusion").map(Origin::Fusion)
            } else if other.is_empty() {
                Err(format!("empty origin — {FORMS}"))
            } else {
                Err(format!("{FORMS}; got `{other}`"))
            }
        }
    }
}

/// How a `Satisfy` line reads against the verification-bullet shape
/// (`archi/requirements/self-hosting/verifications-forgive-the-dash.md`).
enum BulletShape {
    /// Ordinary prose: a `*` bullet, a non-bullet, or a line whose opening
    /// token is not a verification tag (a mistyped `tests` stays here).
    Prose,
    /// A counted verification — `- <tag> <sep> …`, `<sep>` any of `-`/`–`/`—`.
    Verification,
    /// Opens with a verification tag yet misses the shape: a located
    /// diagnostic, never a silent slide into prose.
    TagShapedMiss,
}

/// A verification entry: a trailing `Satisfy` bullet tagged by variant
/// (`archi/requirements/spec-docs/satisfaction-is-a-checked-claim.md`). The
/// separator after the tag is forgiven — ASCII hyphen, en-dash and em-dash
/// all count the same — and a bullet that opens with a tag yet still misses
/// the shape reads as a diagnostic, not prose
/// (`archi/requirements/self-hosting/verifications-forgive-the-dash.md`).
fn verification_bullet(line: &str) -> BulletShape {
    let Some(rest) = line.trim_start().strip_prefix("- ") else {
        return BulletShape::Prose;
    };
    for tag in ["test", "type-level"] {
        let Some(after) = rest.strip_prefix(tag) else {
            continue;
        };
        // A word-continuation character means a longer word — `tests`,
        // `test-driven` — which stays prose; anything else (space, end of
        // line, or punctuation) opens with the bare tag, so a would-be
        // verification can no longer void in silence.
        if after
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            continue;
        }
        let sep = after.trim_start();
        return if matches!(sep.chars().next(), Some('-' | '\u{2013}' | '\u{2014}')) {
            BulletShape::Verification
        } else {
            BulletShape::TagShapedMiss
        };
    }
    BulletShape::Prose
}

/// Subrequirement headings paired with their parent section's slug (`None`
/// = the file's own requirement).
type SubHeadings<'a> = Vec<(&'a Heading, Option<String>)>;

/// The fixed section order of a requirement document: `System Context`,
/// `Satisfy`, then the subrequirement tree. Returns the `Satisfy` heading
/// and the subrequirement headings; `None` when the shape deviates
/// (reported).
fn requirement_sections<'a>(
    doc: &'a MdDoc,
    file: &str,
    diags: &mut Vec<DocDiagnostic>,
) -> Option<(&'a Heading, SubHeadings<'a>)> {
    let hs = &doc.headings;
    let opens_right = hs.len() >= 2
        && hs[0].level == 2
        && hs[0].text == "System Context"
        && hs[1].level == 2
        && hs[1].text == "Satisfy";
    if !opens_right {
        let line = hs.first().map_or(doc.name_line, |h| h.line);
        diags.push(DocDiagnostic::new(
            "E_DOC",
            "a requirement's sections are `System Context`, `Satisfy`, then subrequirements",
            file,
            line,
        ));
        return None;
    }
    let mut subs: Vec<(&Heading, Option<String>)> = Vec::new();
    // Stack of open subrequirement sections: (heading level, slug).
    let mut stack: Vec<(usize, String)> = Vec::new();
    for h in &hs[2..] {
        if RESERVED.contains(&h.text.as_str()) {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!(
                    "reserved heading `{}` reappears — sections come in fixed order",
                    h.text
                ),
                file,
                h.line,
            ));
            return None;
        }
        while stack.last().is_some_and(|(l, _)| *l >= h.level) {
            stack.pop();
        }
        let parent_level = stack.last().map_or(1, |(l, _)| *l);
        if h.level != parent_level + 1 {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                "a subrequirement nests exactly one level deeper than its parent",
                file,
                h.line,
            ));
            return None;
        }
        subs.push((h, stack.last().map(|(_, s)| s.clone())));
        stack.push((h.level, slugify(&h.text)));
    }
    Some((&hs[1], subs))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(line: &str) -> &'static str {
        match verification_bullet(line) {
            BulletShape::Verification => "verification",
            BulletShape::TagShapedMiss => "miss",
            BulletShape::Prose => "prose",
        }
    }

    #[test]
    fn the_separator_is_forgiven() {
        // ASCII hyphen, en-dash and em-dash after the tag each count as one.
        assert_eq!(shape("- test - registers a burst, logins stay fast"), "verification");
        assert_eq!(shape("- test \u{2013} en-dash counts"), "verification");
        assert_eq!(shape("- test \u{2014} em-dash counts"), "verification");
        assert_eq!(shape("- type-level - the port is declared"), "verification");
        assert_eq!(shape("- type-level \u{2014} the type checks"), "verification");
    }

    #[test]
    fn a_tag_shaped_miss_is_a_diagnostic_not_prose() {
        // Opens with a verification tag but misses the shape — never a silent
        // slide into prose (`verifications-forgive-the-dash`).
        assert_eq!(shape("- test the store never holds plaintext"), "miss");
        assert_eq!(shape("- test: the store never holds plaintext"), "miss");
        assert_eq!(shape("- test"), "miss");
        assert_eq!(shape("- type-level checks the port"), "miss");
    }

    #[test]
    fn a_mistyped_tag_or_star_bullet_stays_prose() {
        // A `*` bullet, a non-bullet, or a word that merely starts with the
        // tag letters stays ordinary prose, unaffected.
        assert_eq!(shape("* test \u{2014} a star bullet is prose"), "prose");
        assert_eq!(shape("- tests \u{2014} a mistyped tag is prose"), "prose");
        assert_eq!(shape("- tested \u{2014} still prose"), "prose");
        assert_eq!(shape("- testing the boundaries here"), "prose");
        assert_eq!(shape("plain prose, no bullet"), "prose");
        assert_eq!(shape("- a plain requirement note"), "prose");
    }
}
