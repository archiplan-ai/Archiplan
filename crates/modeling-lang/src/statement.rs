//! Statements: JSON objects discriminated by their `stmt` field.
//!
//! JSON is the language's concrete syntax. The compact pseudo-syntax
//! (`def node Payments;`, `Service type_of Payments;`) is render-only —
//! [`Statement::pseudo`] produces it for human eyes; nothing parses it.

use std::fmt;

use serde::de::Error as _;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::error::{ErrorCode, LangError};

/// What a `define` / `redefine` statement defines. The subject key — `node`,
/// `view`, `rel` or `conn` — names the element (a path for nodes, a name for
/// everything else); the definition's parameters are sibling fields.
#[derive(Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum Definition {
    /// `"node": <path>` — the prefix names the (existing) container.
    Node { path: String },
    /// `"view": <name>` — a view has no definition body.
    View { name: String },
    /// `"rel": <name>` plus `trans` (optional), `directed`, `source`, `target`.
    Rel {
        name: String,
        trans: bool,
        directed: bool,
        source: PatternExpr,
        target: PatternExpr,
    },
    /// `"conn": <name>` plus `directed`, `source`, `carrier` (ternary types
    /// only), `target`.
    Conn {
        name: String,
        directed: bool,
        source: PatternExpr,
        carrier: Option<PatternExpr>,
        target: PatternExpr,
    },
}

impl Definition {
    /// Pseudo-syntax without the leading verb or trailing `;`:
    /// `node Payments`, `rel trans of_sort := * -> *`.
    fn pseudo(&self) -> String {
        match self {
            Definition::Node { path } => format!("node {path}"),
            Definition::View { name } => format!("view {name}"),
            Definition::Rel {
                name,
                trans,
                directed,
                source,
                target,
            } => {
                let trans = if *trans { "trans " } else { "" };
                let arrow = if *directed { "->" } else { "<->" };
                format!(
                    "rel {trans}{name} := {} {arrow} {}",
                    source.pseudo_slot(),
                    target.pseudo_slot()
                )
            }
            Definition::Conn {
                name,
                directed,
                source,
                carrier,
                target,
            } => {
                let arrow = if *directed { "->" } else { "<->" };
                let carrier = match carrier {
                    Some(c) => c.pseudo_slot(),
                    None => String::new(),
                };
                format!(
                    "conn {name} := {} {carrier}{arrow} {}",
                    source.pseudo_slot(),
                    target.pseudo_slot()
                )
            }
        }
    }
}

impl Serialize for Definition {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        match self {
            Definition::Node { path } => m.serialize_entry("node", path)?,
            Definition::View { name } => m.serialize_entry("view", name)?,
            Definition::Rel {
                name,
                trans,
                directed,
                source,
                target,
            } => {
                m.serialize_entry("rel", name)?;
                if *trans {
                    m.serialize_entry("trans", &true)?;
                }
                m.serialize_entry("directed", directed)?;
                m.serialize_entry("source", source)?;
                m.serialize_entry("target", target)?;
            }
            Definition::Conn {
                name,
                directed,
                source,
                carrier,
                target,
            } => {
                m.serialize_entry("conn", name)?;
                m.serialize_entry("directed", directed)?;
                m.serialize_entry("source", source)?;
                if let Some(c) = carrier {
                    m.serialize_entry("carrier", c)?;
                }
                m.serialize_entry("target", target)?;
            }
        }
        m.end()
    }
}

impl<'de> Deserialize<'de> for Definition {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct NodeBody {
            node: String,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ViewBody {
            view: String,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RelBody {
            rel: String,
            #[serde(default)]
            trans: bool,
            directed: bool,
            source: PatternExpr,
            target: PatternExpr,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ConnBody {
            conn: String,
            directed: bool,
            source: PatternExpr,
            carrier: Option<PatternExpr>,
            target: PatternExpr,
        }

        let v = Value::deserialize(d)?;
        let subjects: Vec<&str> = match v.as_object() {
            Some(obj) => ["node", "view", "rel", "conn"]
                .into_iter()
                .filter(|k| obj.contains_key(*k))
                .collect(),
            None => return Err(D::Error::custom("a definition is a JSON object")),
        };
        let de = D::Error::custom;
        match subjects.as_slice() {
            ["node"] => {
                let b: NodeBody = serde_json::from_value(v).map_err(de)?;
                Ok(Definition::Node { path: b.node })
            }
            ["view"] => {
                let b: ViewBody = serde_json::from_value(v).map_err(de)?;
                Ok(Definition::View { name: b.view })
            }
            ["rel"] => {
                let b: RelBody = serde_json::from_value(v).map_err(de)?;
                Ok(Definition::Rel {
                    name: b.rel,
                    trans: b.trans,
                    directed: b.directed,
                    source: b.source,
                    target: b.target,
                })
            }
            ["conn"] => {
                let b: ConnBody = serde_json::from_value(v).map_err(de)?;
                Ok(Definition::Conn {
                    name: b.conn,
                    directed: b.directed,
                    source: b.source,
                    carrier: b.carrier,
                    target: b.target,
                })
            }
            _ => Err(D::Error::custom(
                "a definition names exactly one subject: `node`, `view`, `rel` or `conn`",
            )),
        }
    }
}

/// A shape/routing pattern as written in a statement:
/// `"*"`, `{ "node": P }` or `{ "anchor": P, "rel": R }`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PatternExpr {
    /// Matches any node.
    Any,
    /// Matches exactly the named node.
    Exact {
        /// Absolute path of the node.
        node: String,
    },
    /// Matches any node `x` such that `anchor rel x`.
    Classified {
        /// Absolute path of the classifying node.
        anchor: String,
        /// Name of the classifying relation type.
        rel: String,
    },
}

impl PatternExpr {
    /// Pseudo-syntax without surrounding parentheses: `*`, `OrderId`,
    /// `Service type_of *`.
    fn pseudo_bare(&self) -> String {
        match self {
            PatternExpr::Any => "*".to_string(),
            PatternExpr::Exact { node } => node.clone(),
            PatternExpr::Classified { anchor, rel } => format!("{anchor} {rel} *"),
        }
    }

    /// Pseudo-syntax as a shape slot: `*`, `(OrderId)`, `(Service type_of *)`.
    fn pseudo_slot(&self) -> String {
        match self {
            PatternExpr::Any => "*".to_string(),
            _ => format!("({})", self.pseudo_bare()),
        }
    }
}

impl fmt::Display for PatternExpr {
    /// The pattern in the spec's pseudo-syntax, without parentheses:
    /// `*`, `OrderId`, `Service type_of *`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.pseudo_bare())
    }
}

impl Serialize for PatternExpr {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            PatternExpr::Any => s.serialize_str("*"),
            PatternExpr::Exact { node } => {
                let mut m = serde_json::Map::new();
                m.insert("node".into(), Value::String(node.clone()));
                Value::Object(m).serialize(s)
            }
            PatternExpr::Classified { anchor, rel } => {
                let mut m = serde_json::Map::new();
                m.insert("anchor".into(), Value::String(anchor.clone()));
                m.insert("rel".into(), Value::String(rel.clone()));
                Value::Object(m).serialize(s)
            }
        }
    }
}

impl<'de> Deserialize<'de> for PatternExpr {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        match &v {
            Value::String(s) if s == "*" => Ok(PatternExpr::Any),
            Value::String(s) => Err(D::Error::custom(format!(
                "a pattern string must be \"*\", got {s:?}"
            ))),
            Value::Object(map) => {
                let keys: Vec<&str> = map.keys().map(String::as_str).collect();
                let get = |k: &str| -> Result<String, D::Error> {
                    map.get(k)
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .ok_or_else(|| {
                            D::Error::custom(format!("pattern field `{k}` must be a string"))
                        })
                };
                match keys.as_slice() {
                    ["node"] => Ok(PatternExpr::Exact { node: get("node")? }),
                    ["anchor", "rel"] | ["rel", "anchor"] => Ok(PatternExpr::Classified {
                        anchor: get("anchor")?,
                        rel: get("rel")?,
                    }),
                    _ => Err(D::Error::custom(
                        "a pattern object is {\"node\": ...} or {\"anchor\": ..., \"rel\": ...}",
                    )),
                }
            }
            _ => Err(D::Error::custom(
                "a pattern is \"*\", {\"node\": ...} or {\"anchor\": ..., \"rel\": ...}",
            )),
        }
    }
}

/// A connection or application end: a node and a port on it.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct End {
    /// The node — an absolute path, except an application's inner end, which
    /// is a bare child name of the delegating node.
    pub node: String,
    /// The port name; created on first use.
    pub port: String,
}

/// The kind of an edge: the relation/connection/application trichotomy of
/// `requirements/modeling-lang/modeling-lang.md#kinds`. Used as a `query`
/// filter and as edge metadata in query results.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs)]
pub enum EdgeKind {
    Relation,
    Connection,
    Application,
}

impl EdgeKind {
    /// The kind's lowercase name, as written in statements.
    pub fn name(self) -> &'static str {
        match self {
            EdgeKind::Relation => "relation",
            EdgeKind::Connection => "connection",
            EdgeKind::Application => "application",
        }
    }
}

/// One statement of the language. See
/// `requirements/modeling-lang/modeling-lang.md#statements`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "stmt", rename_all = "kebab-case")]
#[allow(missing_docs)]
pub enum Statement {
    /// Idempotent creation: creates if absent, no-ops on an identical
    /// restatement, rejects a divergent existing definition.
    Define(Definition),
    /// Replacement: the element must exist. A node redefine empties its
    /// scope; a type redefine replaces the shape.
    Redefine(Definition),
    RelEdge {
        rel: String,
        source: String,
        target: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        views: Vec<String>,
    },
    ConnEdge {
        conn: String,
        source: End,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        carrier: Option<String>,
        target: End,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        views: Vec<String>,
    },
    App {
        node: String,
        port: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        route: Option<PatternExpr>,
        inner: End,
    },
    Rename {
        node: String,
        to: String,
    },
    Delete {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        node: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        edge: Option<Box<Statement>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rel: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        conn: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        view: Option<String>,
    },
    Untag {
        edge: Box<Statement>,
        views: Vec<String>,
    },
    /// Subgraph query (`requirements/modeling-lang/queries.md`): composable
    /// filters, each optional; an absent filter does not restrict. An empty
    /// list is the most restrictive filter (matches nothing), not an absent
    /// one — `"scopes": []` means "the top level only".
    Query {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        types: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kinds: Option<Vec<EdgeKind>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        views: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        scopes: Option<Vec<String>>,
    },
    Check {
        #[serde(default, rename = "in", skip_serializing_if = "Vec::is_empty")]
        in_views: Vec<String>,
    },
}

/// Allowed fields per statement kind, used for strict schema validation
/// (internally-tagged serde enums cannot deny unknown fields themselves).
/// `define` / `redefine` are keyed by their subject in [`definition_keys`].
fn allowed_keys(kind: &str) -> Option<&'static [&'static str]> {
    Some(match kind {
        "rel-edge" => &["stmt", "rel", "source", "target", "views"],
        "conn-edge" => &["stmt", "conn", "source", "carrier", "target", "views"],
        "app" => &["stmt", "node", "port", "route", "inner"],
        "rename" => &["stmt", "node", "to"],
        "delete" => &["stmt", "node", "edge", "rel", "conn", "view"],
        "untag" => &["stmt", "edge", "views"],
        "query" => &["stmt", "types", "kinds", "views", "scopes"],
        "check" => &["stmt", "in"],
        _ => return None,
    })
}

/// Allowed fields of a `define` / `redefine`, decided by the one subject key
/// present; `None` when the subject is missing or ambiguous.
fn definition_keys(obj: &serde_json::Map<String, Value>) -> Option<&'static [&'static str]> {
    let subjects: Vec<&str> = ["node", "view", "rel", "conn"]
        .into_iter()
        .filter(|k| obj.contains_key(*k))
        .collect();
    Some(match subjects.as_slice() {
        ["node"] => &["stmt", "node"],
        ["view"] => &["stmt", "view"],
        ["rel"] => &["stmt", "rel", "trans", "directed", "source", "target"],
        ["conn"] => &["stmt", "conn", "directed", "source", "carrier", "target"],
        _ => return None,
    })
}

fn parse_err(message: String, subject: &Value) -> LangError {
    LangError::new(ErrorCode::Parse, message).with_subject(subject.clone())
}

fn validate_keys(value: &Value) -> Result<&str, LangError> {
    let obj = value
        .as_object()
        .ok_or_else(|| parse_err("a statement is a JSON object".into(), value))?;
    let kind = obj
        .get("stmt")
        .and_then(Value::as_str)
        .ok_or_else(|| parse_err("missing or non-string `stmt` field".into(), value))?;
    let allowed = match kind {
        "define" | "redefine" => definition_keys(obj).ok_or_else(|| {
            parse_err(
                format!("`{kind}` names exactly one subject: `node`, `view`, `rel` or `conn`"),
                value,
            )
        })?,
        _ => allowed_keys(kind)
            .ok_or_else(|| parse_err(format!("unknown statement kind `{kind}`"), value))?,
    };
    for key in obj.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(parse_err(
                format!("unknown field `{key}` for a `{kind}` statement"),
                value,
            ));
        }
    }
    if let Some(edge) = obj.get("edge") {
        let inner = validate_keys(edge)?;
        if !matches!(inner, "rel-edge" | "conn-edge" | "app") {
            return Err(parse_err(
                format!("`edge` must restate an edge statement, got `{inner}`"),
                value,
            ));
        }
    }
    Ok(kind)
}

/// Parse one statement from a JSON value, with strict schema validation:
/// unknown kinds and fields, ill-typed fields, an `edge` field that does not
/// restate an edge, a `delete` without exactly one target, a `define` /
/// `redefine` without exactly one subject, and `redefine` of a view are all
/// `E_PARSE`.
pub fn parse_statement(value: &Value) -> Result<Statement, LangError> {
    validate_keys(value)?;
    let stmt: Statement = serde_path_to_error::deserialize(value).map_err(
        |e: serde_path_to_error::Error<serde_json::Error>| {
            let path = e.path().to_string();
            let msg = if path == "." {
                format!("{}", e.inner())
            } else {
                format!("{} (at `{path}`)", e.inner())
            };
            parse_err(msg, value)
        },
    )?;
    match &stmt {
        Statement::Delete {
            node,
            edge,
            rel,
            conn,
            view,
        } => {
            let targets = [
                node.is_some(),
                edge.is_some(),
                rel.is_some(),
                conn.is_some(),
                view.is_some(),
            ];
            if targets.iter().filter(|t| **t).count() != 1 {
                return Err(parse_err(
                    "`delete` takes exactly one target: `node`, `edge`, `rel`, `conn` or `view`"
                        .into(),
                    value,
                ));
            }
        }
        Statement::Redefine(Definition::View { .. }) => {
            return Err(parse_err(
                "a view has no definition body; `redefine` does not apply (`define` only)".into(),
                value,
            ));
        }
        _ => {}
    }
    Ok(stmt)
}

fn views_suffix(views: &[String]) -> String {
    if views.is_empty() {
        String::new()
    } else {
        format!(" in {}", views.join(", "))
    }
}

impl Statement {
    /// The statement as a JSON value.
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).expect("statements serialize")
    }

    /// Pseudo-syntax without the trailing `;` — used to embed an edge
    /// restatement inside `delete`/`untag` renderings.
    fn pseudo_bare(&self) -> String {
        let s = self.pseudo();
        s.strip_suffix(';').unwrap_or(&s).to_string()
    }

    /// Render the statement in the spec's illustrative pseudo-syntax
    /// (`def node Payments;`). Presentation only — nothing parses it.
    pub fn pseudo(&self) -> String {
        match self {
            Statement::Define(d) => format!("def {};", d.pseudo()),
            Statement::Redefine(d) => format!("redefine {};", d.pseudo()),
            Statement::RelEdge {
                rel,
                source,
                target,
                views,
            } => {
                format!("{source} {rel} {target}{};", views_suffix(views))
            }
            Statement::ConnEdge {
                conn,
                source,
                carrier,
                target,
                views,
            } => {
                let carrier = match carrier {
                    Some(c) => format!("({c})"),
                    None => String::new(),
                };
                format!(
                    "{}({}) {conn}{carrier} {}({}){};",
                    source.node,
                    source.port,
                    target.node,
                    target.port,
                    views_suffix(views)
                )
            }
            Statement::App {
                node,
                port,
                route,
                inner,
            } => {
                let route = match route {
                    Some(r) => format!("({})", r.pseudo_bare()),
                    None => String::new(),
                };
                format!("{node}.{port}{route} = {}({});", inner.node, inner.port)
            }
            Statement::Rename { node, to } => format!("rename {node} {to};"),
            Statement::Delete {
                node,
                edge,
                rel,
                conn,
                view,
            } => {
                if let Some(n) = node {
                    format!("delete {n};")
                } else if let Some(e) = edge {
                    format!("delete {};", e.pseudo_bare())
                } else if let Some(r) = rel {
                    format!("delete rel {r};")
                } else if let Some(c) = conn {
                    format!("delete conn {c};")
                } else if let Some(v) = view {
                    format!("delete view {v};")
                } else {
                    "delete;".to_string()
                }
            }
            Statement::Untag { edge, views } => {
                format!("untag {}{};", edge.pseudo_bare(), views_suffix(views))
            }
            Statement::Query {
                types,
                kinds,
                views,
                scopes,
            } => {
                let seg = |kw: &str, items: Option<Vec<String>>| match items {
                    Some(items) => format!(" {kw} ({})", items.join(", ")),
                    None => String::new(),
                };
                format!(
                    "query{}{}{}{};",
                    seg("types", types.clone()),
                    seg(
                        "kinds",
                        kinds
                            .as_ref()
                            .map(|ks| ks.iter().map(|k| k.name().to_string()).collect())
                    ),
                    seg("scopes", scopes.clone()),
                    seg("in", views.clone()),
                )
            }
            Statement::Check { in_views } => format!("check{};", views_suffix(in_views)),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.pseudo())
    }
}
