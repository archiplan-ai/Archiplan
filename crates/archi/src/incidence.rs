//! Incidence analysis over stress sessions (`requirements/scoring/incidence.md`):
//! session selection, per-version affects expansion, and rendering.
//!
//! The core matrix and findings live in [`modeling_lang::Model::incidence`];
//! this module owns everything session-shaped. Each stressor's affects
//! expand against *its own* session's pinned version — reconstructed from
//! the archive, so closed sessions re-validate exactly as
//! `requirements/stressing.md#compile` demands — and the expanded term
//! paths join against the **frame**: the newest pinned version in scope.
//! Invariants — the `satisfied-by` claims of intent-origin requirements —
//! ride along for the compound-vulnerability findings.

use std::collections::BTreeMap;
use std::path::Path;

use modeling_lang::{
    IncidenceConfig, IncidenceFinding, IncidenceReport, IncidenceRow, Invariant, Model, Severity,
    StressOutcome,
};
use serde_json::{Value, json};

use crate::docs::{self, schema};
use crate::versions;

/// What to analyze.
#[derive(Default)]
pub struct Options {
    /// One session by slug; `None` picks the open session, then the
    /// latest-closed one.
    pub session: Option<String>,
    /// All sessions pressing this version or a later one.
    pub since: Option<String>,
    /// Drop stressors with no outcome yet.
    pub exclude_pending: bool,
    /// Core tunables.
    pub config: IncidenceConfig,
}

/// One session of the scope, for the report envelope.
#[derive(Debug)]
pub struct SessionRef {
    /// The session's slug.
    pub slug: String,
    /// The version it presses on.
    pub version: String,
    /// The version whose save closed it; `None` while open.
    pub closed: Option<String>,
}

/// A finished analysis: the scope's sessions, the frame version and the
/// core report.
#[derive(Debug)]
pub struct Analysis {
    /// Sessions in scope, pinned-version order.
    pub sessions: Vec<SessionRef>,
    /// The frame version the matrix joined against.
    pub frame: String,
    /// The core report.
    pub report: IncidenceReport,
}

fn version_num(id: &str) -> Option<u64> {
    id.strip_prefix('v').and_then(|s| s.parse().ok())
}

/// Run the analysis. `model` is the compiled live tree — the doc sources
/// cross-check against it exactly as `archi check` does, and a stress tree
/// that does not compile refuses to be analyzed.
pub fn analyze(root: &Path, model: &Model, opts: &Options) -> Result<Analysis, String> {
    let (tree, doc) = docs::load(root, model);
    let stress_diags: Vec<String> = doc
        .diagnostics
        .iter()
        .filter(|d| d.file.starts_with("archi/stress/"))
        .map(ToString::to_string)
        .collect();
    if !stress_diags.is_empty() {
        return Err(format!(
            "the stress tree does not compile — fix these first:\n{}",
            stress_diags.join("\n")
        ));
    }

    let archive = versions::Archive::open(root)?
        .ok_or("no versions saved — a stress session needs a version to press on")?;

    // Selection. The diagnostics gate above guarantees every session pins
    // an archived version.
    let mut selected: Vec<&schema::Session> = match (&opts.session, &opts.since) {
        (Some(slug), None) => vec![
            tree.sessions
                .iter()
                .find(|s| &s.slug == slug)
                .ok_or_else(|| format!("no stress session `{slug}`"))?,
        ],
        (None, Some(since)) => {
            let bound = version_num(since)
                .ok_or_else(|| format!("`{since}` is not a version id (vNNNN)"))?;
            let hits: Vec<&schema::Session> = tree
                .sessions
                .iter()
                .filter(|s| pinned_num(s).is_some_and(|n| n >= bound))
                .collect();
            if hits.is_empty() {
                return Err(format!("no session presses {since} or a later version"));
            }
            hits
        }
        (None, None) => {
            let pick = tree.sessions.iter().find(|s| s.open()).or_else(|| {
                tree.sessions
                    .iter()
                    .filter(|s| !s.open())
                    .max_by_key(|s| s.closed.as_ref().and_then(|(v, _)| version_num(v)))
            });
            vec![pick.ok_or("no stress sessions to analyze")?]
        }
        (Some(_), Some(_)) => return Err("--session and --since are mutually exclusive".into()),
    };
    selected.sort_by_key(|s| (pinned_num(s), s.slug.clone()));

    // The frame: the newest pinned version in scope. Every distinct pinned
    // version compiles once; affects expand against their own.
    let frame_id = selected
        .iter()
        .filter_map(|s| s.version.as_ref().map(|(v, _)| v.clone()))
        .max_by_key(|v| version_num(v))
        .ok_or("the scope pins no version")?;
    let mut models: BTreeMap<String, Model> = BTreeMap::new();
    for s in &selected {
        let (v, _) = s.version.as_ref().expect("diagnostics gate sound sessions");
        if !models.contains_key(v) {
            let ws = docs::compile_version(root, &archive, v)
                .map_err(|e| format!("version `{v}` of session `{}`: {e}", s.slug))?;
            models.insert(v.clone(), ws.model().clone());
        }
    }

    let mut rows: Vec<IncidenceRow> = Vec::new();
    let mut sessions: Vec<SessionRef> = Vec::new();
    for s in &selected {
        let (v, _) = s.version.as_ref().expect("diagnostics gate sound sessions");
        let version = &models[v];
        let mut stressors: Vec<&schema::Stressor> = tree
            .stressors
            .iter()
            .filter(|st| st.session == s.slug)
            .collect();
        stressors.sort_by_key(|st| st.slug.as_str());
        for st in stressors {
            let outcome = match st.outcome.expect("diagnostics gate sound stressors") {
                schema::Outcome::Pending => StressOutcome::Pending,
                schema::Outcome::Surviving => StressOutcome::Surviving,
                schema::Outcome::Breaking => StressOutcome::Breaking,
            };
            if opts.exclude_pending && outcome == StressOutcome::Pending {
                continue;
            }
            let (affects, _) = st.affects.as_ref().expect("diagnostics gate sound stressors");
            let mut terms = Vec::new();
            for p in affects {
                terms.extend(version.term_surface(p).ok_or_else(|| {
                    format!(
                        "affects of `{}` names no element `{p}` of version `{v}` (E_MODEL_REF)",
                        st.slug
                    )
                })?);
            }
            rows.push(IncidenceRow {
                id: st.slug.clone(),
                terms,
                outcome,
            });
        }
        sessions.push(SessionRef {
            slug: s.slug.clone(),
            version: v.clone(),
            closed: s.closed.as_ref().and_then(|(c, _)| {
                if c.is_empty() { None } else { Some(c.clone()) }
            }),
        });
    }

    // Invariants: the satisfaction claims of intent-origin requirements —
    // promises the initial problem statement extracted.
    let invariants: Vec<Invariant> = tree
        .requirements
        .iter()
        .filter_map(|r| {
            let f = r.fields.as_ref()?;
            let (schema::Origin::Intent, _) = f.origin.as_ref()? else {
                return None;
            };
            let (elements, _) = f.satisfied_by.as_ref()?;
            (!elements.is_empty()).then(|| Invariant {
                id: r.slug.clone(),
                elements: elements.clone(),
            })
        })
        .collect();

    let report = models[&frame_id].incidence(&rows, &invariants, &opts.config);
    Ok(Analysis {
        sessions,
        frame: frame_id,
        report,
    })
}

fn pinned_num(s: &schema::Session) -> Option<u64> {
    s.version.as_ref().and_then(|(v, _)| version_num(v))
}

/// The findings that pass `--kind` / `--min-severity`.
pub fn filter<'a>(
    findings: &'a [IncidenceFinding],
    kinds: &[String],
    min_severity: Option<Severity>,
) -> Vec<&'a IncidenceFinding> {
    findings
        .iter()
        .filter(|f| kinds.is_empty() || kinds.iter().any(|k| k == f.kind.name()))
        .filter(|f| min_severity.is_none_or(|min| f.severity >= min))
        .collect()
}

/// The default output: the matrix (unless `no_matrix`) and the findings,
/// as text for humans.
pub fn render_human(a: &Analysis, no_matrix: bool, findings: &[&IncidenceFinding]) -> String {
    let mut out = String::new();
    match a.sessions.as_slice() {
        [s] => {
            out.push_str(&format!(
                "incidence — session `{}` presses {}{}\n",
                s.slug,
                s.version,
                match &s.closed {
                    Some(c) => format!(", closed by {c}"),
                    None => " (open)".to_string(),
                }
            ));
        }
        many => {
            out.push_str(&format!(
                "incidence — {} sessions, frame {}\n",
                many.len(),
                a.frame
            ));
            for s in many {
                out.push_str(&format!(
                    "  {}  presses {}{}\n",
                    s.slug,
                    s.version,
                    match &s.closed {
                        Some(c) => format!(", closed by {c}"),
                        None => " (open)".to_string(),
                    }
                ));
            }
        }
    }
    let scope = &a.report.scope;
    out.push_str(&format!(
        "S×N = {}×{} · K_hyper = {:.3}\n",
        scope.stressor_count, scope.component_count, scope.k_hyper
    ));

    let m = &a.report.matrix;
    if !no_matrix && !m.stressors.is_empty() && !m.components.is_empty() {
        out.push('\n');
        let width = m.stressors.iter().map(String::len).max().unwrap_or(0);
        let ruler: String = (1..=m.components.len())
            .map(|j| char::from_digit((j % 10) as u32, 10).expect("digit"))
            .collect();
        out.push_str(&format!("  {:width$}  {ruler}\n", ""));
        for (i, slug) in m.stressors.iter().enumerate() {
            let cells: String = m.rows[i]
                .iter()
                .map(|&c| if c == 1 { '■' } else { '·' })
                .collect();
            out.push_str(&format!(
                "  {slug:width$}  {cells}  {}\n",
                m.outcomes[i].describe()
            ));
        }
        out.push('\n');
        let digits = m.components.len().to_string().len();
        for (j, path) in m.components.iter().enumerate() {
            out.push_str(&format!("  {:>digits$} {path}\n", j + 1));
        }
    }

    out.push('\n');
    if findings.is_empty() {
        out.push_str("no findings\n");
    } else {
        out.push_str("findings\n");
        for f in findings {
            out.push_str(&format!("  {f}\n"));
        }
    }
    if !a.report.warnings.is_empty() {
        out.push_str("\nwarnings\n");
        for w in &a.report.warnings {
            out.push_str(&format!("  {}: {}\n", w.code, w.message));
        }
    }
    out
}

/// The full report as a JSON envelope: the core report plus the scope's
/// sessions and the frame version.
pub fn to_json(a: &Analysis, no_matrix: bool, findings: &[&IncidenceFinding]) -> Value {
    let mut v = serde_json::to_value(&a.report).expect("serializes");
    v["findings"] = serde_json::to_value(findings).expect("serializes");
    if no_matrix {
        v.as_object_mut().expect("report is an object").remove("matrix");
    }
    v["frame"] = json!(a.frame);
    v["sessions"] = Value::Array(
        a.sessions
            .iter()
            .map(|s| json!({ "session": s.slug, "version": s.version, "closed": s.closed }))
            .collect(),
    );
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use modeling_lang::IncidenceKind;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    const MODEL: &str = "def node AuthService:\n  port login\ndef node CredStore:\n  port creds\ndef node Legacy\nService type_of AuthService\ndef conn store := * -> *\nAuthService.login store CredStore.creds\n";

    fn temp_project() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-incidence-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(dir.join("archi/src")).unwrap();
        fs::write(
            dir.join("archi.toml"),
            "[project]\nname = \"t\"\npreset = \"default\"\n",
        )
        .unwrap();
        fs::write(dir.join("archi/src").join("model.arch"), MODEL).unwrap();
        dir
    }

    fn put(root: &Path, rel_path: &str, text: &str) {
        let path = root.join(rel_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn live_model(root: &Path) -> Model {
        modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("test model failed to compile:\n{}", f.render()))
            .workspace
            .model()
            .clone()
    }

    fn save_version(root: &Path, note: &str) -> String {
        let model = live_model(root);
        match versions::save(root, &model, note).unwrap() {
            versions::Saved::Written { id, .. } => id,
            versions::Saved::Unchanged { latest } => panic!("unchanged at {latest}"),
        }
    }

    fn session(root: &Path, slug: &str, version: &str, closed: &str) {
        put(
            root,
            &format!("archi/stress/{slug}/{slug}.md"),
            &format!(
                "---\nversion: {version}\nclosed: {closed}\n---\n\n# {slug}\n\nA round.\n"
            ),
        );
    }

    fn stressor(root: &Path, session: &str, slug: &str, affects: &str, outcome: &str) {
        let resolution = if outcome == "pending" { "" } else { "\nHeld.\n" };
        put(
            root,
            &format!("archi/stress/{session}/{slug}.md"),
            &format!(
                "---\naffects: [{affects}]\noutcome: {outcome}\n---\n\n# {slug}\n\nPresses.\n\n## Attractor\n\nBends.\n\n## Resolution\n{resolution}"
            ),
        );
    }

    fn analyze_with(root: &Path, opts: &Options) -> Result<Analysis, String> {
        analyze(root, &live_model(root), opts)
    }

    #[test]
    fn affects_expand_against_the_pinned_version_not_the_live_tree() {
        let root = temp_project();
        let v1 = save_version(&root, "first");
        // The live tree grows another Service instance the pinned version
        // does not have: the row must not widen to it.
        fs::write(
            root.join("archi/src/model.arch"),
            format!("{MODEL}def node RateLimiter\nService type_of RateLimiter\n"),
        )
        .unwrap();
        session(&root, "round", &v1, "");
        stressor(&root, "round", "press", "Service", "surviving");
        let a = analyze_with(&root, &Options::default()).unwrap();
        assert_eq!(a.frame, "v0001");
        assert_eq!(a.sessions.len(), 1);
        assert_eq!(a.sessions[0].closed, None);
        assert_eq!(
            a.report.matrix.components,
            ["AuthService", "CredStore", "Legacy"]
        );
        assert_eq!(a.report.matrix.rows, [[1, 0, 0]]);

        // The renders carry the story: header, ruler, cells, envelope.
        let findings = filter(&a.report.findings, &[], None);
        let text = render_human(&a, false, &findings);
        assert!(text.contains("session `round` presses v0001 (open)"), "{text}");
        assert!(text.contains("press  ■··  surviving"), "{text}");
        let v = to_json(&a, true, &findings);
        assert_eq!(v["frame"], "v0001");
        assert_eq!(v["sessions"][0]["session"], "round");
        assert!(v.get("matrix").is_none());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn since_joins_sessions_against_the_newest_frame_and_keeps_drops_visible() {
        let root = temp_project();
        let v1 = save_version(&root, "first");
        // v2 drops `Legacy` and grows `TenantKeyer`.
        fs::write(
            root.join("archi/src/model.arch"),
            MODEL.replace("def node Legacy\n", "def node TenantKeyer\n"),
        )
        .unwrap();
        let v2 = save_version(&root, "second");
        session(&root, "one", &v1, &v2);
        stressor(&root, "one", "legacy-press", "Legacy, AuthService", "surviving");
        session(&root, "two", &v2, "");
        stressor(&root, "two", "keyer-press", "TenantKeyer", "pending");

        // Alone, the closed session frames its own version: `Legacy` is a
        // column and nothing drops.
        let one = analyze_with(
            &root,
            &Options {
                session: Some("one".to_string()),
                ..Options::default()
            },
        )
        .unwrap();
        assert_eq!(one.frame, "v0001");
        assert!(one.report.scope.dropped.is_empty());
        assert!(one.report.matrix.components.contains(&"Legacy".to_string()));

        // Since v0001, the newest pinned version frames the join; the
        // affected term v0002 no longer knows stays visible as a drop.
        let both = analyze_with(
            &root,
            &Options {
                since: Some("v0001".to_string()),
                ..Options::default()
            },
        )
        .unwrap();
        assert_eq!(both.frame, "v0002");
        assert_eq!(
            both.sessions.iter().map(|s| s.slug.as_str()).collect::<Vec<_>>(),
            ["one", "two"]
        );
        assert_eq!(both.report.scope.dropped["legacy-press"], ["Legacy"]);
        assert!(both.report.warnings.iter().any(|w| w.code == "DROPPED_AFFECTS"));
        let components = &both.report.matrix.components;
        assert!(components.contains(&"TenantKeyer".to_string()));
        assert!(!components.contains(&"Legacy".to_string()));

        // No selector: the open session wins; excluding pending empties it.
        let open = analyze_with(&root, &Options::default()).unwrap();
        assert_eq!(open.sessions[0].slug, "two");
        let empty = analyze_with(
            &root,
            &Options {
                exclude_pending: true,
                ..Options::default()
            },
        )
        .unwrap();
        assert!(empty.report.warnings.iter().any(|w| w.code == "NO_STRESSORS"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn invariants_are_the_intent_origin_claims() {
        let root = temp_project();
        let v1 = save_version(&root, "first");
        put(root.as_path(), "archi/requirements/area/area.md", "# Area\n\nThe problem.\n");
        let requirement = |name: &str, origin: &str, satisfied_by: &str| {
            format!(
                "---\nkind: functional\norigin: {origin}\nsatisfied-by: [{satisfied_by}]\ndeferred:\n---\n\n# {name}\n\nA claim.\n\n## System Context\n\n## Satisfy\n\nHeld by the named elements.\n"
            )
        };
        put(
            root.as_path(),
            "archi/requirements/area/guarded.md",
            &requirement("Guarded", "intent", "AuthService, CredStore"),
        );
        // The same claim with a stressor origin is an answer to pressure,
        // not an initial promise: it must not seed compound analysis.
        put(
            root.as_path(),
            "archi/requirements/area/derived.md",
            &requirement("Derived", "stressor(press-c)", "AuthService, CredStore"),
        );
        session(&root, "round", &v1, "");
        stressor(&root, "round", "press-a", "AuthService", "surviving");
        stressor(&root, "round", "press-b", "CredStore", "surviving");
        stressor(&root, "round", "press-c", "CredStore", "breaking");
        let a = analyze_with(&root, &Options::default()).unwrap();
        let compounds: Vec<(&str, &str)> = a
            .report
            .findings
            .iter()
            .filter_map(|f| match &f.kind {
                IncidenceKind::CompoundVulnerability {
                    stressors,
                    invariant,
                    ..
                } => Some((invariant.as_str(), stressors[0].as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(compounds, [("guarded", "press-a")]);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn selection_and_validation_speak_plainly() {
        let root = temp_project();
        let v1 = save_version(&root, "first");
        assert_eq!(
            analyze_with(&root, &Options::default()).unwrap_err(),
            "no stress sessions to analyze"
        );

        // A closed session re-validates on analysis: its affects must
        // resolve in the version it pressed on.
        fs::write(
            root.join("archi/src/model.arch"),
            format!("{MODEL}def node Extra\n"),
        )
        .unwrap();
        let v2 = save_version(&root, "second");
        session(&root, "done", &v1, &v2);
        stressor(&root, "done", "phantom", "Ghost", "surviving");
        let err = analyze_with(&root, &Options::default()).unwrap_err();
        assert!(
            err.contains("names no element `Ghost` of version `v0001`"),
            "{err}"
        );

        // An open session's bad affects are already compile diagnostics:
        // the analysis refuses the whole stress tree.
        session(&root, "fresh", &v2, "");
        stressor(&root, "fresh", "misref", "Ghost", "pending");
        let err = analyze_with(&root, &Options::default()).unwrap_err();
        assert!(err.contains("the stress tree does not compile"), "{err}");
        assert!(err.contains("E_MODEL_REF"), "{err}");
        fs::remove_dir_all(&root).unwrap();

        let root = temp_project();
        save_version(&root, "first");
        assert_eq!(
            analyze_with(
                &root,
                &Options {
                    session: Some("ghost".to_string()),
                    ..Options::default()
                }
            )
            .unwrap_err(),
            "no stress session `ghost`"
        );
        assert_eq!(
            analyze_with(
                &root,
                &Options {
                    since: Some("nonsense".to_string()),
                    ..Options::default()
                }
            )
            .unwrap_err(),
            "`nonsense` is not a version id (vNNNN)"
        );
        fs::remove_dir_all(&root).unwrap();
    }
}
