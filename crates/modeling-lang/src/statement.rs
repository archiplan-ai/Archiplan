//! Statements: JSON objects discriminated by their `stmt` field.
//!
//! JSON is the statement API's concrete syntax. [`Statement::pseudo`]
//! renders a statement in the `.arch` surface syntax
//! (`def node Payments`, `Service type_of Payments`) — creation statements
//! render as valid source text, so dumps paste back into modules; reads
//! render display-only.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::Error as _;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::definition;
use crate::error::{ErrorCode, LangError};

/// What a `define` statement defines. The subject key — `node`,
/// `view`, `rel` or `conn` — names the element (a path for nodes, a name for
/// everything else); the definition's parameters are sibling fields.
///
/// Every subject takes an optional `doc`: the element's definition — one
/// sentence of identity prose ([`crate::definition`]), part of the element's
/// identity like every other parameter. An omitted `doc` makes no claim; a
/// present one is compared against the stored definition.
#[derive(Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum Definition {
    /// `"node": <path>` — the prefix names the (existing) container — plus
    /// `ports` (optional): the node's declared ports. An omitted `ports`
    /// field makes no claim about the port set; a present one is compared
    /// exactly against the node's declared ports. `port_docs` (optional)
    /// maps declared ports to their definitions, and requires `ports`.
    Node {
        path: String,
        ports: Option<Vec<String>>,
        doc: Option<String>,
        port_docs: Option<BTreeMap<String, String>>,
    },
    /// `"view": <name>` — a view's definition body is its `doc` alone.
    View { name: String, doc: Option<String> },
    /// `"rel": <name>` plus `trans` (optional), `directed`, `source`, `target`.
    Rel {
        name: String,
        trans: bool,
        directed: bool,
        source: PatternExpr,
        target: PatternExpr,
        doc: Option<String>,
    },
    /// `"conn": <name>` plus `directed`, `source`, `carrier` (forward-lane
    /// carried slot), `rev_carrier` (reverse-lane carried slot, directed
    /// types only), `target`.
    Conn {
        name: String,
        directed: bool,
        source: PatternExpr,
        carrier: Option<PatternExpr>,
        rev_carrier: Option<PatternExpr>,
        target: PatternExpr,
        doc: Option<String>,
    },
}

/// A definition rendered as its trailing comment, empty when absent.
fn doc_suffix(doc: &Option<String>) -> String {
    match doc {
        Some(d) => format!(" // {d}"),
        None => String::new(),
    }
}

impl Definition {
    /// Surface syntax without the leading verb: `node Payments`,
    /// `rel trans of_sort := * -> *`. A node with declared ports renders as
    /// its block form, across lines. Definitions render as trailing
    /// comments on their defining lines — the attach pass reads the same
    /// position back, so dumps round-trip.
    fn pseudo(&self) -> String {
        match self {
            Definition::Node {
                path,
                ports,
                doc,
                port_docs,
            } => match ports {
                None => format!("node {path}{}", doc_suffix(doc)),
                Some(ps) => {
                    let mut out = format!("node {path}:{}", doc_suffix(doc));
                    for p in ps {
                        out.push_str("\n  port ");
                        out.push_str(p);
                        if let Some(d) = port_docs.as_ref().and_then(|m| m.get(p)) {
                            out.push_str(" // ");
                            out.push_str(d);
                        }
                    }
                    out
                }
            },
            Definition::View { name, doc } => format!("view {name}{}", doc_suffix(doc)),
            Definition::Rel {
                name,
                trans,
                directed,
                source,
                target,
                doc,
            } => {
                let trans = if *trans { "trans " } else { "" };
                let arrow = if *directed { "->" } else { "<->" };
                format!(
                    "rel {trans}{name} := {} {arrow} {}{}",
                    source.pseudo_slot(),
                    target.pseudo_slot(),
                    doc_suffix(doc)
                )
            }
            Definition::Conn {
                name,
                directed,
                source,
                carrier,
                rev_carrier,
                target,
                doc,
            } => {
                let mut lanes = String::from(if *directed { "->" } else { "<->" });
                if let Some(c) = carrier {
                    lanes.push_str(&c.pseudo_slot());
                }
                if let Some(rc) = rev_carrier {
                    lanes.push_str(", <-");
                    lanes.push_str(&rc.pseudo_slot());
                }
                format!(
                    "conn {name} := {} {lanes} {}{}",
                    source.pseudo_slot(),
                    target.pseudo_slot(),
                    doc_suffix(doc)
                )
            }
        }
    }
}

impl Serialize for Definition {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        match self {
            Definition::Node {
                path,
                ports,
                doc,
                port_docs,
            } => {
                m.serialize_entry("node", path)?;
                if let Some(ps) = ports {
                    m.serialize_entry("ports", ps)?;
                }
                if let Some(d) = doc {
                    m.serialize_entry("doc", d)?;
                }
                if let Some(pd) = port_docs {
                    m.serialize_entry("port_docs", pd)?;
                }
            }
            Definition::View { name, doc } => {
                m.serialize_entry("view", name)?;
                if let Some(d) = doc {
                    m.serialize_entry("doc", d)?;
                }
            }
            Definition::Rel {
                name,
                trans,
                directed,
                source,
                target,
                doc,
            } => {
                m.serialize_entry("rel", name)?;
                if *trans {
                    m.serialize_entry("trans", &true)?;
                }
                m.serialize_entry("directed", directed)?;
                m.serialize_entry("source", source)?;
                m.serialize_entry("target", target)?;
                if let Some(d) = doc {
                    m.serialize_entry("doc", d)?;
                }
            }
            Definition::Conn {
                name,
                directed,
                source,
                carrier,
                rev_carrier,
                target,
                doc,
            } => {
                m.serialize_entry("conn", name)?;
                m.serialize_entry("directed", directed)?;
                m.serialize_entry("source", source)?;
                if let Some(c) = carrier {
                    m.serialize_entry("carrier", c)?;
                }
                if let Some(rc) = rev_carrier {
                    m.serialize_entry("rev_carrier", rc)?;
                }
                m.serialize_entry("target", target)?;
                if let Some(d) = doc {
                    m.serialize_entry("doc", d)?;
                }
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
            ports: Option<Vec<String>>,
            doc: Option<String>,
            port_docs: Option<BTreeMap<String, String>>,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ViewBody {
            view: String,
            doc: Option<String>,
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
            doc: Option<String>,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ConnBody {
            conn: String,
            directed: bool,
            source: PatternExpr,
            carrier: Option<PatternExpr>,
            rev_carrier: Option<PatternExpr>,
            target: PatternExpr,
            doc: Option<String>,
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
                Ok(Definition::Node {
                    path: b.node,
                    ports: b.ports,
                    doc: b.doc,
                    port_docs: b.port_docs,
                })
            }
            ["view"] => {
                let b: ViewBody = serde_json::from_value(v).map_err(de)?;
                Ok(Definition::View {
                    name: b.view,
                    doc: b.doc,
                })
            }
            ["rel"] => {
                let b: RelBody = serde_json::from_value(v).map_err(de)?;
                Ok(Definition::Rel {
                    name: b.rel,
                    trans: b.trans,
                    directed: b.directed,
                    source: b.source,
                    target: b.target,
                    doc: b.doc,
                })
            }
            ["conn"] => {
                let b: ConnBody = serde_json::from_value(v).map_err(de)?;
                Ok(Definition::Conn {
                    name: b.conn,
                    directed: b.directed,
                    source: b.source,
                    carrier: b.carrier,
                    rev_carrier: b.rev_carrier,
                    target: b.target,
                    doc: b.doc,
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
    /// The pattern in an already-delimited position: `*`, `OrderId`,
    /// `Service type_of *`.
    fn pseudo_bare(&self) -> String {
        match self {
            PatternExpr::Any => "*".to_string(),
            PatternExpr::Exact { node } => node.clone(),
            PatternExpr::Classified { anchor, rel } => format!("{anchor} {rel} *"),
        }
    }

    /// The pattern as a shape slot: `*`, `OrderId`, `(Service type_of *)` —
    /// parenthesized only where juxtaposition demands delimiting.
    fn pseudo_slot(&self) -> String {
        match self {
            PatternExpr::Any | PatternExpr::Exact { .. } => self.pseudo_bare(),
            PatternExpr::Classified { .. } => format!("({})", self.pseudo_bare()),
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rev_carrier: Option<String>,
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
    /// Subgraph query (`requirements/modeling-lang/queries.md`): composable
    /// filters, each optional; an absent filter does not restrict. An empty
    /// list is the most restrictive filter (matches nothing), not an absent
    /// one — `"scopes": []` means "the top level only". `carriers` keeps
    /// connection edges carrying one of the named nodes (directly or via a
    /// classifying type); `edge_types` keeps edges of the named rel/conn
    /// types — applications are untyped and never pass it.
    Query {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        types: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kinds: Option<Vec<EdgeKind>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        views: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        scopes: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        carriers: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        edge_types: Option<Vec<String>>,
    },
    Check {
        #[serde(default, rename = "in", skip_serializing_if = "Vec::is_empty")]
        in_views: Vec<String>,
    },
}

/// Allowed fields per statement kind, used for strict schema validation
/// (internally-tagged serde enums cannot deny unknown fields themselves).
/// `define` is keyed by its subject in [`definition_keys`].
fn allowed_keys(kind: &str) -> Option<&'static [&'static str]> {
    Some(match kind {
        "rel-edge" => &["stmt", "rel", "source", "target", "views"],
        "conn-edge" => &[
            "stmt",
            "conn",
            "source",
            "carrier",
            "rev_carrier",
            "target",
            "views",
        ],
        "app" => &["stmt", "node", "port", "route", "inner"],
        "query" => &[
            "stmt",
            "types",
            "kinds",
            "views",
            "scopes",
            "carriers",
            "edge_types",
        ],
        "check" => &["stmt", "in"],
        _ => return None,
    })
}

/// Allowed fields of a `define`, decided by the one subject key
/// present; `None` when the subject is missing or ambiguous.
fn definition_keys(obj: &serde_json::Map<String, Value>) -> Option<&'static [&'static str]> {
    let subjects: Vec<&str> = ["node", "view", "rel", "conn"]
        .into_iter()
        .filter(|k| obj.contains_key(*k))
        .collect();
    Some(match subjects.as_slice() {
        ["node"] => &["stmt", "node", "ports", "doc", "port_docs"],
        ["view"] => &["stmt", "view", "doc"],
        ["rel"] => &["stmt", "rel", "trans", "directed", "source", "target", "doc"],
        ["conn"] => &[
            "stmt",
            "conn",
            "directed",
            "source",
            "carrier",
            "rev_carrier",
            "target",
            "doc",
        ],
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
        "define" => definition_keys(obj).ok_or_else(|| {
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
    Ok(kind)
}

/// Normalize and validate the definition prose of a `define`, and check
/// that `port_docs` names only declared ports. The same shared rule guards
/// the source attach pass and the engine — every door stores only
/// normalized text, so renders round-trip byte-stably.
fn validate_docs(d: &mut Definition) -> Result<(), String> {
    fn check(doc: &mut Option<String>) -> Result<(), String> {
        if let Some(text) = doc {
            let normalized = definition::normalize(text);
            definition::validate(&normalized)?;
            *text = normalized;
        }
        Ok(())
    }
    match d {
        Definition::Node {
            ports,
            doc,
            port_docs,
            ..
        } => {
            check(doc)?;
            if let Some(pd) = port_docs {
                let Some(ps) = ports else {
                    return Err(
                        "`port_docs` requires `ports`: definitions attach to declared ports"
                            .into(),
                    );
                };
                for (port, text) in pd.iter_mut() {
                    if !ps.contains(port) {
                        return Err(format!(
                            "`port_docs` names `{port}`, which is not in `ports`"
                        ));
                    }
                    let normalized = definition::normalize(text);
                    definition::validate(&normalized)?;
                    *text = normalized;
                }
            }
        }
        Definition::View { doc, .. }
        | Definition::Rel { doc, .. }
        | Definition::Conn { doc, .. } => check(doc)?,
    }
    Ok(())
}

/// Parse one statement from a JSON value, with strict schema validation:
/// unknown kinds and fields, ill-typed fields, a `define` without exactly
/// one subject, and invalid definition prose are all `E_PARSE`.
pub fn parse_statement(value: &Value) -> Result<Statement, LangError> {
    validate_keys(value)?;
    let mut stmt: Statement = serde_path_to_error::deserialize(value).map_err(
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
        Statement::Define(Definition::Conn {
            directed: false,
            rev_carrier: Some(_),
            ..
        }) => {
            return Err(parse_err(
                "an undirected connection type has no lanes; `rev_carrier` requires `directed`"
                    .into(),
                value,
            ));
        }
        Statement::Define(Definition::Node {
            ports: Some(ps), ..
        }) => {
            if let Some(dup) = ps
                .iter()
                .enumerate()
                .find_map(|(i, p)| ps[..i].contains(p).then_some(p))
            {
                return Err(parse_err(
                    format!("duplicate port `{dup}` in a node definition"),
                    value,
                ));
            }
        }
        _ => {}
    }
    if let Statement::Define(d) = &mut stmt {
        validate_docs(d).map_err(|m| parse_err(m, value))?;
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

    /// Render the statement in the `.arch` surface syntax. Creation
    /// statements — the ones dumps are made of — render as valid source
    /// text, pasteable into a module; reads have no surface form and render
    /// display-only.
    pub fn pseudo(&self) -> String {
        match self {
            Statement::Define(d) => format!("def {}", d.pseudo()),
            Statement::RelEdge {
                rel,
                source,
                target,
                views,
            } => {
                format!("{source} {rel} {target}{}", views_suffix(views))
            }
            Statement::ConnEdge {
                conn,
                source,
                carrier,
                rev_carrier,
                target,
                views,
            } => {
                let carriers = match (carrier, rev_carrier) {
                    (Some(c), Some(rc)) => format!("(->{c}, <-{rc})"),
                    (Some(c), None) => format!("({c})"),
                    (None, Some(rc)) => format!("(<-{rc})"),
                    (None, None) => String::new(),
                };
                format!(
                    "{}.{} {conn}{carriers} {}.{}{}",
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
                format!("{node}.{port}{route} = {}.{}", inner.node, inner.port)
            }
            Statement::Query {
                types,
                kinds,
                views,
                scopes,
                carriers,
                edge_types,
            } => {
                let seg = |kw: &str, items: Option<Vec<String>>| match items {
                    Some(items) => format!(" {kw} ({})", items.join(", ")),
                    None => String::new(),
                };
                format!(
                    "query{}{}{}{}{}{}",
                    seg("types", types.clone()),
                    seg(
                        "kinds",
                        kinds
                            .as_ref()
                            .map(|ks| ks.iter().map(|k| k.name().to_string()).collect())
                    ),
                    seg("carriers", carriers.clone()),
                    seg("edge_types", edge_types.clone()),
                    seg("scopes", scopes.clone()),
                    seg("in", views.clone()),
                )
            }
            Statement::Check { in_views } => format!("check{}", views_suffix(in_views)),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.pseudo())
    }
}
