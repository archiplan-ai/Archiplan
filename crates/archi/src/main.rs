//! `archi` — a thin runner for the `.arch` source-format compiler
//! (`requirements/cli.md`, `requirements/modeling-lang/source-format.md`),
//! the NKP landscape analysis (`requirements/scoring/nkp.md`), the incidence
//! analysis (`requirements/scoring/incidence.md`), the version archive
//! (`requirements/versioning.md`) and the doc sources — intents,
//! requirements, stress sessions (`requirements/requirements.md`,
//! `requirements/stressing.md`), compiled and cross-checked by `check`.
//!
//! ```text
//! archi check [--project <dir>] [--json]
//! archi build [--project <dir>] [--emit-batch <file|->]
//! archi nkp   [--project <dir>] [--regime | --hotspots | --corridors] [--top | --scope <path>]
//!             [--exclude '<src> <rel> <dst>']... [--only <edge-type>]...
//!             [--tau-p <f>] [--tau-b <f>] [--neutrality degree|uniform] [--global-p <f>]
//! archi incidence [--project <dir>] [--session <slug> | --since <id>] [--exclude-pending]
//!             [--all-terms] [--json | --matrix | --k-hyper | --findings] [--no-matrix]
//!             [--kind <kind>]... [--min-severity info|warn|alert]
//!             [--tau-j <f>] [--tau-d <f>] [--depth <n>] [--path-limit <n>]
//! archi version save -m <note> | anchor | list | show <id> | diff <a> <b> | current
//!             [--project <dir>]
//! archi link add <spec[@ver]> <file[#symbol]> --kind literal|indirect
//! archi link ls [--spec <ref>] [--evidence] [--json]
//! archi link verify [--spec <ref>] [--since <rev>] [--json]
//! archi link confirm <id> | rm <id>... | rm --spec <ref> --yes
//! archi link repin <id> [--to <file[#symbol]>]
//! archi link capture --task <TASK>
//! archi link audit [--scope <path>] [--since <rev>] [--prune] [--json]
//! archi plan use <name> | repin | show [--json] | verify [--json]
//! archi plan task add <node> [--desc <text>]
//! archi plan start | next | current-wave | close | reset
//! archi read  [<request.json> | -] [--at <id>]
//! archi query [--scope <path>]... [--type <path>]... [--kind <k>]... [--view <v>]...
//!             [--carrier <path>]... [--edge-type <name>]... [--top] [--at <id>]
//! ```
//!
//! Every verb locates its project by precedence: `--project`, then the
//! nearest `archi.toml` upward from the working directory. A project of
//! `.arch` files is compiled fresh each run — the source is the model, and
//! the only source of truth: the CLI offers no JSON editing of the model;
//! mutation is a text edit and a recompile.

mod docs;
mod incidence;
mod links;
mod plans;
mod versions;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use modeling_lang::source::{Compiled, compile_project, find_project_root};
use modeling_lang::{
    ExcludePattern, Finding, IncidenceConfig, Model, Neutrality, NkpConfig, NkpScope, Severity,
    Statement, Workspace,
};
use serde_json::{Value, json};

const USAGE: &str = "usage:
  archi check [--project <dir>] [--json]
  archi build [--project <dir>] [--emit-batch <file|->]
  archi nkp   [--project <dir>] [--regime | --hotspots | --corridors] [--top | --scope <path>]
              [--exclude '<src> <rel> <dst>']... [--only <edge-type>]...
              [--tau-p <f>] [--tau-b <f>] [--neutrality degree|uniform] [--global-p <f>]
  archi incidence [--project <dir>] [--session <slug> | --since <id>] [--exclude-pending]
              [--all-terms] [--json | --matrix | --k-hyper | --findings] [--no-matrix]
              [--kind <kind>]... [--min-severity info|warn|alert]
              [--tau-j <f>] [--tau-d <f>] [--depth <n>] [--path-limit <n>]
  archi version save -m <note> [--project <dir>]
  archi version anchor [--project <dir>]
  archi version list [--project <dir>]
  archi version show <id> [--project <dir>]
  archi version diff <a> <b> [--project <dir>]
  archi version current [--project <dir>]
  archi link add <spec[@ver]> <file[#symbol]> --kind literal|indirect [--project <dir>]
  archi link ls [--spec <ref>] [--evidence] [--json] [--project <dir>]
  archi link verify [--spec <ref>] [--since <rev>] [--json] [--project <dir>]
  archi link confirm <id> | rm <id>... | rm --spec <ref> --yes [--project <dir>]
  archi link repin <id> [--to <file[#symbol]>] [--project <dir>]
  archi link capture --task <TASK> [--project <dir>]
  archi link audit [--scope <path>] [--since <rev>] [--prune] [--json] [--project <dir>]
  archi plan use <name> | repin | show [--json] | verify [--json] [--project <dir>]
  archi plan task add <node> [--desc <text>] [--project <dir>]
  archi plan start | next | current-wave | close | reset [--project <dir>]
  archi read [<request.json> | -] [--at <id>] [--project <dir>]
  archi query [--scope <path>]... [--type <path>]... [--kind <k>]... [--view <v>]...
              [--carrier <path>]... [--edge-type <name>]... [--top] [--at <id>] [--project <dir>]";

struct Args {
    verb: String,
    project: Option<String>,
    emit_batch: Option<String>,
    json: bool,
    regime: bool,
    hotspots: bool,
    corridors: bool,
    top: bool,
    scope: Option<String>,
    exclude: Vec<String>,
    only: Vec<String>,
    tau_p: Option<f64>,
    tau_b: Option<f64>,
    neutrality: Option<String>,
    global_p: Option<f64>,
    session: Option<String>,
    since: Option<String>,
    exclude_pending: bool,
    all_terms: bool,
    no_matrix: bool,
    matrix: bool,
    k_hyper: bool,
    findings: bool,
    kind: Vec<String>,
    min_severity: Option<String>,
    tau_j: Option<f64>,
    tau_d: Option<f64>,
    depth: Option<usize>,
    path_limit: Option<usize>,
    message: Option<String>,
    spec: Option<String>,
    kind_flag: Option<String>,
    to: Option<String>,
    task: Option<String>,
    desc: Option<String>,
    at: Option<String>,
    types: Vec<String>,
    views: Vec<String>,
    scopes: Vec<String>,
    carriers: Vec<String>,
    edge_types: Vec<String>,
    evidence: bool,
    yes: bool,
    prune: bool,
    positional: Vec<String>,
}

fn usage_err(msg: &str) -> ExitCode {
    eprintln!("archi: {msg}\n{USAGE}");
    ExitCode::from(2)
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut args = Args {
        verb: String::new(),
        project: None,
        emit_batch: None,
        json: false,
        regime: false,
        hotspots: false,
        corridors: false,
        top: false,
        scope: None,
        exclude: Vec::new(),
        only: Vec::new(),
        tau_p: None,
        tau_b: None,
        neutrality: None,
        global_p: None,
        session: None,
        since: None,
        exclude_pending: false,
        all_terms: false,
        no_matrix: false,
        matrix: false,
        k_hyper: false,
        findings: false,
        kind: Vec::new(),
        min_severity: None,
        tau_j: None,
        tau_d: None,
        depth: None,
        path_limit: None,
        message: None,
        spec: None,
        kind_flag: None,
        to: None,
        task: None,
        desc: None,
        at: None,
        types: Vec::new(),
        views: Vec::new(),
        scopes: Vec::new(),
        carriers: Vec::new(),
        edge_types: Vec::new(),
        evidence: false,
        yes: false,
        prune: false,
        positional: Vec::new(),
    };
    let mut it = argv.iter().peekable();
    let Some(verb) = it.next() else {
        return Err("missing command".into());
    };
    args.verb = verb.clone();
    let value = |it: &mut std::iter::Peekable<std::slice::Iter<String>>,
                 flag: &str|
     -> Result<String, String> {
        it.next().cloned().ok_or(format!("{flag} needs a value"))
    };
    let float = |v: String, flag: &str| -> Result<f64, String> {
        v.parse().map_err(|_| format!("{flag} needs a number"))
    };
    let int = |v: String, flag: &str| -> Result<usize, String> {
        v.parse().map_err(|_| format!("{flag} needs an integer"))
    };
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => args.json = true,
            "--regime" => args.regime = true,
            "--hotspots" => args.hotspots = true,
            "--corridors" => args.corridors = true,
            "--top" => args.top = true,
            "--exclude-pending" => args.exclude_pending = true,
            "--all-terms" => args.all_terms = true,
            "--no-matrix" => args.no_matrix = true,
            "--matrix" => args.matrix = true,
            "--k-hyper" => args.k_hyper = true,
            "--findings" => args.findings = true,
            "--project" => args.project = Some(value(&mut it, "--project")?),
            "--emit-batch" => args.emit_batch = Some(value(&mut it, "--emit-batch")?),
            // `query` composes repeatable scopes; nkp and link audit keep
            // the single --scope.
            "--scope" if args.verb == "query" => {
                args.scopes.push(value(&mut it, "--scope")?)
            }
            "--scope" => args.scope = Some(value(&mut it, "--scope")?),
            "--at" => args.at = Some(value(&mut it, "--at")?),
            "--type" => args.types.push(value(&mut it, "--type")?),
            "--view" => args.views.push(value(&mut it, "--view")?),
            "--carrier" => args.carriers.push(value(&mut it, "--carrier")?),
            "--edge-type" => args.edge_types.push(value(&mut it, "--edge-type")?),
            "--exclude" => args.exclude.push(value(&mut it, "--exclude")?),
            "--only" => args.only.push(value(&mut it, "--only")?),
            "--neutrality" => args.neutrality = Some(value(&mut it, "--neutrality")?),
            "--session" => args.session = Some(value(&mut it, "--session")?),
            "--since" => args.since = Some(value(&mut it, "--since")?),
            "--evidence" => args.evidence = true,
            "--yes" => args.yes = true,
            "--prune" => args.prune = true,
            "--spec" => args.spec = Some(value(&mut it, "--spec")?),
            "--to" => args.to = Some(value(&mut it, "--to")?),
            "--task" => args.task = Some(value(&mut it, "--task")?),
            "--desc" => args.desc = Some(value(&mut it, "--desc")?),
            // `link` reads the singular `--kind literal|indirect`; the
            // incidence filter keeps the repeatable list.
            "--kind" if args.verb == "link" => {
                args.kind_flag = Some(value(&mut it, "--kind")?)
            }
            "--kind" => args.kind.push(value(&mut it, "--kind")?),
            "--min-severity" => args.min_severity = Some(value(&mut it, "--min-severity")?),
            "--tau-p" => args.tau_p = Some(float(value(&mut it, "--tau-p")?, "--tau-p")?),
            "--tau-b" => args.tau_b = Some(float(value(&mut it, "--tau-b")?, "--tau-b")?),
            "--tau-j" => args.tau_j = Some(float(value(&mut it, "--tau-j")?, "--tau-j")?),
            "--tau-d" => args.tau_d = Some(float(value(&mut it, "--tau-d")?, "--tau-d")?),
            "--depth" => args.depth = Some(int(value(&mut it, "--depth")?, "--depth")?),
            "--path-limit" => {
                args.path_limit = Some(int(value(&mut it, "--path-limit")?, "--path-limit")?)
            }
            "--global-p" => {
                args.global_p = Some(float(value(&mut it, "--global-p")?, "--global-p")?)
            }
            "-m" | "--message" => args.message = Some(value(&mut it, "-m")?),
            other if other.starts_with("--") => return Err(format!("unknown flag `{other}`")),
            other if matches!(args.verb.as_str(), "version" | "link" | "plan" | "read") => {
                args.positional.push(other.to_string())
            }
            other => return Err(format!("unexpected argument `{other}`")),
        }
    }
    Ok(args)
}

/// Locate the project: `--project <dir>`, then the nearest `archi.toml`
/// upward from the working directory.
fn locate_project(args: &Args) -> Result<PathBuf, String> {
    if let Some(p) = &args.project {
        return Ok(PathBuf::from(p));
    }
    std::env::current_dir()
        .ok()
        .and_then(|d| find_project_root(&d))
        .ok_or_else(|| {
            format!(
                "`{}` needs a project: pass --project <dir> or run inside one (archi.toml)",
                args.verb
            )
        })
}

/// Compile a project, reporting diagnostics as `file:line:col: CODE: message`
/// lines (or a structured JSON envelope with `--json`).
fn compile_or_report(root: &Path, json_out: bool) -> Result<Compiled, ExitCode> {
    match compile_project(root) {
        Ok(c) => Ok(c),
        Err(f) => {
            if json_out {
                let diags: Vec<Value> = f
                    .diagnostics
                    .iter()
                    .map(|d| {
                        let mut o = json!({ "code": d.code, "message": d.message });
                        if let Some(s) = d.span {
                            let (file, line, col) = f.map.location(s);
                            o["file"] = json!(file);
                            o["line"] = json!(line);
                            o["col"] = json!(col);
                        }
                        o
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &json!({ "status": "error", "diagnostics": diags })
                    )
                    .expect("serializes")
                );
            } else {
                eprintln!("{}", f.render());
            }
            Err(ExitCode::from(1))
        }
    }
}

/// The workspace `check`/`nkp` analyze: the located project, compiled fresh.
fn analysis_workspace(args: &Args) -> Result<Workspace, ExitCode> {
    let root = locate_project(args).map_err(|e| usage_err(&e))?;
    compile_or_report(&root, args.json).map(|c| c.workspace)
}

fn run_check(args: &Args) -> ExitCode {
    let root = match locate_project(args) {
        Ok(r) => r,
        Err(e) => return usage_err(&e),
    };
    let ws = match compile_or_report(&root, args.json) {
        Ok(c) => c.workspace,
        Err(code) => return code,
    };
    let findings: Vec<Finding> = ws.model().check();
    // The version archive is sealed: an edited keyframe, patch or manifest
    // is a compile error, not a finding (requirements/versioning.md).
    let archive_errors = versions::verify_at(&root);
    // Doc sources — intents, requirements, stress sessions — compile with
    // the model and cross-check against it; their errors fail the check,
    // their findings are advisory (requirements/requirements.md#compile).
    let doc = docs::check(&root, ws.model());
    if args.json {
        let mut all: Vec<Value> = findings
            .iter()
            .map(|f| serde_json::to_value(f).expect("serializes"))
            .collect();
        all.extend(
            doc.findings
                .iter()
                .map(|f| serde_json::to_value(f).expect("serializes")),
        );
        let mut envelope = json!({ "status": "ok", "findings": all });
        if !archive_errors.is_empty() {
            envelope["status"] = json!("error");
            envelope["archive"] = json!(
                archive_errors
                    .iter()
                    .map(|m| json!({ "code": "E_ARCHIVE", "message": m }))
                    .collect::<Vec<Value>>()
            );
        }
        if !doc.diagnostics.is_empty() {
            envelope["status"] = json!("error");
            envelope["docs"] = serde_json::to_value(&doc.diagnostics).expect("serializes");
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&envelope).expect("serializes")
        );
    } else {
        if findings.is_empty() && doc.findings.is_empty() {
            println!("no findings");
        } else {
            for f in &findings {
                println!("{f}");
            }
            for f in &doc.findings {
                println!("{f}");
            }
        }
        for e in &archive_errors {
            eprintln!("archi/versions: E_ARCHIVE: {e}");
        }
        for d in &doc.diagnostics {
            eprintln!("{d}");
        }
    }
    if archive_errors.is_empty() && doc.diagnostics.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_version(args: &Args) -> ExitCode {
    let fail = |e: String| -> ExitCode {
        eprintln!("archi: {e}");
        ExitCode::from(1)
    };
    let sub = args.positional.first().map(String::as_str);
    let rest = args.positional.get(1..).unwrap_or_default();
    let root = match locate_project(args) {
        Ok(r) => r,
        Err(e) => return usage_err(&e),
    };
    // Subcommands that read the archive alone; save, anchor and current
    // compile the live tree first — a model that does not compile has no
    // version.
    match (sub, rest) {
        (Some("save"), []) => {
            let Some(note) = args.message.as_deref() else {
                return usage_err("`version save` needs a note: -m <note>");
            };
            let ws = match compile_or_report(&root, false) {
                Ok(c) => c.workspace,
                Err(code) => return code,
            };
            match versions::save(&root, ws.model(), note) {
                Ok(versions::Saved::Written {
                    id,
                    kind,
                    file,
                    bytes,
                }) => {
                    println!(
                        "saved {id} ({}, {bytes} bytes, {}) — {note}",
                        kind.describe(),
                        file.display()
                    );
                    // Saving closes the active stress session, and the
                    // incidence report fires over the finished round
                    // (requirements/versioning.md#versioning--stressing).
                    match docs::close_open_session(&root, &id) {
                        Ok(Some(session)) => {
                            println!("closed stress session `{session}`");
                            fire_incidence(&root, ws.model(), &session);
                        }
                        Ok(None) => {}
                        Err(e) => eprintln!("archi: warning: {e}"),
                    }
                    ExitCode::SUCCESS
                }
                Ok(versions::Saved::Unchanged { latest }) => {
                    // No mint on an unchanged model — but the ceremony
                    // finishes: an open round closes against the current
                    // version and its incidence report fires; the bare
                    // no-op is a success
                    // (requirements/versioning.md#versioning--stressing).
                    match docs::close_open_session(&root, &latest) {
                        Ok(Some(session)) => {
                            println!("nothing to mint: the model is unchanged since {latest}");
                            println!("closed stress session `{session}` at {latest} — {note}");
                            fire_incidence(&root, ws.model(), &session);
                            ExitCode::SUCCESS
                        }
                        Ok(None) => {
                            println!(
                                "nothing to save: the model is unchanged since {latest} and no session is open"
                            );
                            ExitCode::SUCCESS
                        }
                        Err(e) => fail(e),
                    }
                }
                Err(e) => fail(e),
            }
        }
        (Some("anchor"), []) => {
            let ws = match compile_or_report(&root, false) {
                Ok(c) => c.workspace,
                Err(code) => return code,
            };
            match versions::anchor(&root, ws.model()) {
                Ok(versions::Anchored::Recorded { id, commit }) => {
                    println!("anchored {id} at commit {commit}");
                    ExitCode::SUCCESS
                }
                Ok(versions::Anchored::Already { id, commit }) => {
                    println!("{id} is already anchored at commit {commit}");
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            }
        }
        (Some("list"), []) => match versions::Archive::open(&root) {
            Ok(None) => {
                println!("no versions saved");
                ExitCode::SUCCESS
            }
            Ok(Some(archive)) => {
                for e in archive.entries() {
                    println!(
                        "{}  {}  {:8}  {}",
                        e.id,
                        e.created,
                        e.kind.describe(),
                        e.note
                    );
                }
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        },
        (Some("show"), [id]) => match versions::Archive::open(&root) {
            Ok(None) => fail("no versions saved".into()),
            Ok(Some(archive)) => match archive.reconstruct(id) {
                Ok(text) => {
                    print!("{text}");
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            },
            Err(e) => fail(e),
        },
        (Some("diff"), [a, b]) => match versions::Archive::open(&root) {
            Ok(None) => fail("no versions saved".into()),
            Ok(Some(archive)) => {
                match archive
                    .reconstruct(a)
                    .and_then(|from| archive.reconstruct(b).map(|to| (from, to)))
                {
                    Ok((from, to)) => {
                        print!("{}", diffy::create_patch(&from, &to));
                        ExitCode::SUCCESS
                    }
                    Err(e) => fail(e),
                }
            }
            Err(e) => fail(e),
        },
        (Some("current"), []) => {
            let ws = match compile_or_report(&root, false) {
                Ok(c) => c.workspace,
                Err(code) => return code,
            };
            match versions::current(&root, ws.model()) {
                Ok(versions::Current::NoVersions) => println!("no versions saved"),
                Ok(versions::Current::At(id)) => println!("at {id}"),
                Ok(versions::Current::DirtySince(id)) => {
                    println!("dirty: unsaved model changes since {id}")
                }
                Err(e) => return fail(e),
            }
            ExitCode::SUCCESS
        }
        _ => usage_err(
            "`version` takes: save -m <note> | anchor | list | show <id> | diff <a> <b> | current",
        ),
    }
}

fn run_link(args: &Args) -> ExitCode {
    let fail = |e: String| -> ExitCode {
        eprintln!("archi: {e}");
        ExitCode::from(1)
    };
    let pretty = |v: Value| serde_json::to_string_pretty(&v).expect("serializes");
    let root = match locate_project(args) {
        Ok(r) => r,
        Err(e) => return usage_err(&e),
    };
    // Subcommands touching the spec side — add, verify, audit — compile the
    // live tree first; journal-only verbs don't need a compiling model.
    let live_model = || -> Result<modeling_lang::Workspace, ExitCode> {
        compile_or_report(&root, false).map(|c| c.workspace)
    };
    let sub = args.positional.first().map(String::as_str);
    let rest = args.positional.get(1..).unwrap_or_default();
    match (sub, rest) {
        (Some("add"), [spec, code]) => {
            let Some(kind) = args.kind_flag.as_deref().and_then(links::LinkKind::parse) else {
                return usage_err("`link add` needs --kind literal|indirect");
            };
            let ws = match live_model() {
                Ok(ws) => ws,
                Err(code) => return code,
            };
            match links::add(&root, ws.model(), spec, code, kind) {
                Ok(l) => {
                    println!("linked {}", links::render_link(&l));
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            }
        }
        (Some("ls"), []) => match links::ls(&root, args.spec.as_deref(), args.evidence) {
            Ok(live) => {
                if args.json {
                    println!(
                        "{}",
                        pretty(serde_json::to_value(&live).expect("serializes"))
                    );
                } else if live.is_empty() {
                    println!("no links");
                } else {
                    for l in &live {
                        println!("{}", links::render_link(l));
                    }
                }
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        },
        (Some("verify"), []) => {
            let ws = match live_model() {
                Ok(ws) => ws,
                Err(code) => return code,
            };
            let opts = links::VerifyOptions {
                spec: args.spec.clone(),
                since: args.since.clone(),
            };
            match links::verify(&root, ws.model(), &opts) {
                Ok(report) => {
                    if args.json {
                        println!(
                            "{}",
                            pretty(serde_json::to_value(&report).expect("serializes"))
                        );
                    } else {
                        print!("{}", links::render_verify(&report));
                    }
                    if report.failing() {
                        ExitCode::from(1)
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(e) => fail(e),
            }
        }
        (Some("confirm"), [id]) => match links::confirm(&root, id) {
            Ok(l) => {
                println!("asserted {}", links::render_link(&l));
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        },
        (Some("rm"), []) if args.spec.is_some() => {
            if !args.yes {
                return usage_err("`link rm --spec <ref>` retires in bulk: confirm with --yes");
            }
            match links::retire_spec(&root, args.spec.as_deref().expect("checked")) {
                Ok(ids) => {
                    println!("retired {}", ids.join(", "));
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            }
        }
        (Some("rm"), ids) if !ids.is_empty() => match links::retire(&root, ids) {
            Ok(()) => {
                println!("retired {}", ids.join(", "));
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        },
        (Some("repin"), [id]) => match links::repin(&root, id, args.to.as_deref()) {
            Ok(l) => {
                println!("repinned {}", links::render_link(&l));
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        },
        (Some("capture"), []) => {
            let Some(task) = args.task.as_deref() else {
                return usage_err("`link capture` re-runs a task capture: --task <TASK>");
            };
            let ws = match live_model() {
                Ok(ws) => ws,
                Err(code) => return code,
            };
            match links::capture::run_manual(&root, ws.model(), task) {
                Ok(outcome) => {
                    print!("{}", links::capture::render_capture(&outcome));
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            }
        }
        (Some("audit"), []) => {
            let ws = match live_model() {
                Ok(ws) => ws,
                Err(code) => return code,
            };
            let opts = links::AuditOptions {
                since: args.since.clone(),
                scope: args.scope.clone(),
                prune: args.prune,
            };
            match links::audit(&root, ws.model(), &opts) {
                Ok(report) => {
                    if args.json {
                        println!(
                            "{}",
                            pretty(serde_json::to_value(&report).expect("serializes"))
                        );
                    } else {
                        print!("{}", links::render_audit(&report));
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            }
        }
        _ => usage_err(
            "`link` takes: add <spec> <file[#symbol]> --kind <k> | ls | verify | confirm <id> | \
             rm <id>... | rm --spec <ref> --yes | repin <id> [--to <ref>] | capture --task <t> | \
             audit",
        ),
    }
}

/// The workspace a read runs against: the live tree, or — with `--at <id>`
/// — the version reconstructed from the sealed archive.
fn read_workspace(args: &Args, root: &Path) -> Result<Workspace, ExitCode> {
    let Some(id) = args.at.as_deref() else {
        return compile_or_report(root, false).map(|c| c.workspace);
    };
    let fail = |e: String| -> ExitCode {
        eprintln!("archi: {e}");
        ExitCode::from(1)
    };
    match versions::Archive::open(root) {
        Ok(Some(archive)) => docs::compile_version(root, &archive, id).map_err(fail),
        Ok(None) => Err(fail("--at needs a version archive; none saved".into())),
        Err(e) => Err(fail(e)),
    }
}

/// `archi read`: the agent read envelope (`requirements/agent-interface.md`)
/// — one batch of read statements in, the response envelope out, verbatim.
/// Exit codes: 0 ok, 1 a statement failed (`error.index` says which),
/// 2 protocol error (the request itself is bad).
fn run_read(args: &Args) -> ExitCode {
    use std::io::IsTerminal as _;
    let root = match locate_project(args) {
        Ok(r) => r,
        Err(e) => return usage_err(&e),
    };
    let text = match args.positional.as_slice() {
        [] if std::io::stdin().is_terminal() => {
            return usage_err("`read` takes a request file, or `-` / piped stdin");
        }
        [] => std::io::read_to_string(std::io::stdin()),
        [path] if path == "-" => std::io::read_to_string(std::io::stdin()),
        [path] => fs::read_to_string(path),
        _ => return usage_err("`read` takes one request file"),
    };
    let text = match text {
        Ok(t) => t,
        Err(e) => {
            eprintln!("archi: cannot read the request: {e}");
            return ExitCode::from(2);
        }
    };
    // Invalid JSON is a protocol error in the same envelope shape the
    // engine emits — the contract holds before the engine is reached.
    let request: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            let envelope = json!({
                "status": "error",
                "error": {
                    "code": "E_BAD_REQUEST",
                    "message": format!("the request is not valid JSON: {e}"),
                },
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&envelope).expect("serializes")
            );
            return ExitCode::from(2);
        }
    };
    let mut ws = match read_workspace(args, &root) {
        Ok(ws) => ws,
        Err(code) => return code,
    };
    let response = ws.handle(&request);
    println!(
        "{}",
        serde_json::to_string_pretty(&response).expect("serializes")
    );
    match &response.error {
        None => ExitCode::SUCCESS,
        Some(e) if e.index.is_some() => ExitCode::from(1),
        Some(_) => ExitCode::from(2),
    }
}

/// `archi query`: one composed subgraph query — the read envelope's
/// convenience spelling (`requirements/modeling-lang/queries.md`). An
/// absent filter does not restrict; `--top` is the explicit empty scopes
/// filter (the top level only).
fn run_query(args: &Args) -> ExitCode {
    if args.top && !args.scopes.is_empty() {
        return usage_err("--top and --scope are mutually exclusive");
    }
    let root = match locate_project(args) {
        Ok(r) => r,
        Err(e) => return usage_err(&e),
    };
    let mut stmt = json!({ "stmt": "query" });
    if !args.types.is_empty() {
        stmt["types"] = json!(args.types);
    }
    if !args.kind.is_empty() {
        stmt["kinds"] = json!(args.kind);
    }
    if !args.views.is_empty() {
        stmt["views"] = json!(args.views);
    }
    if !args.carriers.is_empty() {
        stmt["carriers"] = json!(args.carriers);
    }
    if !args.edge_types.is_empty() {
        stmt["edge_types"] = json!(args.edge_types);
    }
    if args.top {
        stmt["scopes"] = json!([]);
    } else if !args.scopes.is_empty() {
        stmt["scopes"] = json!(args.scopes);
    }
    let mut ws = match read_workspace(args, &root) {
        Ok(ws) => ws,
        Err(code) => return code,
    };
    let response = ws.handle(&json!({ "statements": [stmt] }));
    if let Some(e) = &response.error {
        eprintln!("archi: {}", e.error.message);
        return ExitCode::from(1);
    }
    let results = response.results.expect("an ok response carries results");
    println!(
        "{}",
        serde_json::to_string_pretty(&results[0]).expect("serializes")
    );
    ExitCode::SUCCESS
}

fn run_plan(args: &Args) -> ExitCode {
    let fail = |e: String| -> ExitCode {
        eprintln!("archi: {e}");
        ExitCode::from(1)
    };
    let pretty = |v: Value| serde_json::to_string_pretty(&v).expect("serializes");
    let root = match locate_project(args) {
        Ok(r) => r,
        Err(e) => return usage_err(&e),
    };
    // Every plan verb resolves against models: the live tree for drift and
    // pinning, the archived pin for structure — so the live tree compiles
    // first, exactly as for `link`.
    let live_model = || -> Result<modeling_lang::Workspace, ExitCode> {
        compile_or_report(&root, false).map(|c| c.workspace)
    };
    let sub = args.positional.first().map(String::as_str);
    let rest = args.positional.get(1..).unwrap_or_default();
    match (sub, rest) {
        (Some("use"), [name]) => {
            let ws = match live_model() {
                Ok(ws) => ws,
                Err(code) => return code,
            };
            match plans::use_plan(&root, ws.model(), name) {
                Ok(plans::Used::Created(p)) => {
                    println!("created plan `{}` @ {} — now current", p.name, p.version);
                    ExitCode::SUCCESS
                }
                Ok(plans::Used::Switched(p)) => {
                    println!("switched to plan `{}` @ {}", p.name, p.version);
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            }
        }
        (Some("repin"), []) => {
            let ws = match live_model() {
                Ok(ws) => ws,
                Err(code) => return code,
            };
            match plans::repin(&root, ws.model()) {
                Ok((p, from)) => {
                    println!(
                        "repinned `{}` {from} → {} — `archi plan verify` shows what moved",
                        p.name, p.version
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            }
        }
        (Some("task"), [add, node]) if add == "add" => {
            match plans::task_add(&root, node, args.desc.as_deref()) {
                Ok(t) => {
                    println!("added {} {} — spec_refs: {}", t.id, t.node, t.spec_refs.join(", "));
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            }
        }
        (Some("verify"), []) => {
            let ws = match live_model() {
                Ok(ws) => ws,
                Err(code) => return code,
            };
            match plans::verify(&root, ws.model()) {
                Ok(report) => {
                    if args.json {
                        println!(
                            "{}",
                            pretty(serde_json::to_value(&report).expect("serializes"))
                        );
                    } else {
                        print!("{}", plans::render_report(&report));
                    }
                    if report.errors.is_empty() {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    }
                }
                Err(e) => fail(e),
            }
        }
        (Some("show"), []) => {
            let ws = match live_model() {
                Ok(ws) => ws,
                Err(code) => return code,
            };
            match plans::show(&root, ws.model()) {
                Ok((plan, report)) => {
                    if args.json {
                        println!(
                            "{}",
                            pretty(json!({ "plan": plan, "report": report }))
                        );
                    } else {
                        print!("{}", plans::render_show(&plan, &report));
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            }
        }
        (Some("start"), []) => {
            let ws = match live_model() {
                Ok(ws) => ws,
                Err(code) => return code,
            };
            match plans::start(&root, ws.model()) {
                Ok((plan, wave1)) => {
                    println!(
                        "started plan `{}` — wave 1 in flight: {}",
                        plan.name,
                        wave1.join(", ")
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            }
        }
        (Some("next"), []) => {
            let ws = match live_model() {
                Ok(ws) => ws,
                Err(code) => return code,
            };
            match plans::next(&root, ws.model()) {
                Ok(outcome) => {
                    if let Some(c) = &outcome.capture {
                        print!("{}", links::capture::render_capture(c));
                    }
                    match outcome.step {
                        plans::Step::Blocked(why) => {
                            eprintln!("archi: {why}");
                            ExitCode::from(1)
                        }
                        plans::Step::Wave { closed, next_tasks } => {
                            println!(
                                "wave {closed} closed — in flight: {}",
                                next_tasks.join(", ")
                            );
                            ExitCode::SUCCESS
                        }
                        plans::Step::Scenarios(scenarios) => {
                            println!("all waves closed — scenarios:");
                            for s in scenarios {
                                println!("  - {s}");
                            }
                            println!("one more `archi plan next` closes the plan");
                            ExitCode::SUCCESS
                        }
                        plans::Step::Done => {
                            println!("DONE");
                            ExitCode::SUCCESS
                        }
                    }
                }
                Err(e) => fail(e),
            }
        }
        (Some("current-wave"), []) => {
            let ws = match live_model() {
                Ok(ws) => ws,
                Err(code) => return code,
            };
            match plans::current_wave(&root, ws.model()) {
                Ok((plan, plans::InFlight::Wave(wave, ids))) => {
                    println!("wave {wave} in flight:");
                    for id in &ids {
                        if let Some(t) = plan.tasks.iter().find(|t| &t.id == id) {
                            println!("  {} {} — {}", t.id, t.node, t.description);
                        }
                    }
                    ExitCode::SUCCESS
                }
                Ok((_, plans::InFlight::ScenarioStep)) => {
                    println!(
                        "all waves closed — the scenario step is pending (`archi plan next`)"
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            }
        }
        (Some("close"), []) => match plans::close(&root) {
            Ok(p) => {
                println!("closed plan `{}`", p.name);
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        },
        (Some("reset"), []) => match plans::reset(&root) {
            Ok(p) => {
                println!("reset plan `{}` to draft", p.name);
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        },
        _ => usage_err(
            "`plan` takes: use <name> | repin | show | verify | task add <node> [--desc <text>] | \
             start | next | current-wave | close | reset",
        ),
    }
}

fn run_build(args: &Args) -> ExitCode {
    let root = match locate_project(args) {
        Ok(r) => r,
        Err(e) => return usage_err(&e),
    };
    let compiled = match compile_or_report(&root, false) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let batch: Vec<Value> = compiled.batch.iter().map(Statement::to_value).collect();
    match args.emit_batch.as_deref() {
        None => println!(
            "ok: {} statements compiled from {}",
            batch.len(),
            root.display()
        ),
        Some("-") => println!(
            "{}",
            serde_json::to_string_pretty(&batch).expect("serializes")
        ),
        Some(path) => {
            if let Err(e) = fs::write(
                path,
                serde_json::to_string_pretty(&batch).expect("serializes"),
            ) {
                eprintln!("archi: cannot write `{path}`: {e}");
                return ExitCode::from(2);
            }
        }
    }
    ExitCode::SUCCESS
}

/// The auto-report over the round a save just closed. Its failure is a
/// warning, never a failed save; `ARCHI_REPORT_JSON=1` switches it to JSON.
fn fire_incidence(root: &Path, model: &Model, session: &str) {
    let opts = incidence::Options {
        session: Some(session.to_string()),
        ..Default::default()
    };
    match incidence::analyze(root, model, &opts) {
        Ok(a) => {
            let findings = incidence::filter(&a.report.findings, &[], None);
            if std::env::var("ARCHI_REPORT_JSON").as_deref() == Ok("1") {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&incidence::to_json(&a, false, &findings))
                        .expect("serializes")
                );
            } else {
                print!("{}", incidence::render_human(&a, false, &findings));
            }
        }
        Err(e) => eprintln!("archi: warning: incidence report: {e}"),
    }
}

fn run_incidence(args: &Args) -> ExitCode {
    if [args.json, args.matrix, args.k_hyper, args.findings]
        .iter()
        .filter(|f| **f)
        .count()
        > 1
    {
        return usage_err("--json, --matrix, --k-hyper and --findings are mutually exclusive");
    }
    if args.no_matrix && args.matrix {
        return usage_err("--no-matrix and --matrix are mutually exclusive");
    }
    if args.session.is_some() && args.since.is_some() {
        return usage_err("--session and --since are mutually exclusive");
    }
    const KINDS: [&str; 5] = [
        "hyperliminal_coupling",
        "stress_hotspot",
        "compound_vulnerability",
        "under_stressed",
        "merge_candidate",
    ];
    for k in &args.kind {
        if !KINDS.contains(&k.as_str()) {
            return usage_err(&format!("--kind is one of {}; got `{k}`", KINDS.join(", ")));
        }
    }
    let min_severity = match args.min_severity.as_deref() {
        None => None,
        Some(s) => match Severity::parse(s) {
            Some(sev) => Some(sev),
            None => {
                return usage_err(&format!(
                    "--min-severity is `info`, `warn` or `alert`, got `{s}`"
                ));
            }
        },
    };
    let root = match locate_project(args) {
        Ok(r) => r,
        Err(e) => return usage_err(&e),
    };
    let ws = match compile_or_report(&root, false) {
        Ok(c) => c.workspace,
        Err(code) => return code,
    };
    let mut config = IncidenceConfig::default();
    if let Some(t) = args.tau_j {
        config.tau_j = t;
    }
    if let Some(t) = args.tau_d {
        config.tau_d = t;
    }
    if let Some(d) = args.depth {
        config.depth = d;
    }
    if let Some(l) = args.path_limit {
        config.path_limit = l;
    }
    config.all_terms = args.all_terms;
    let opts = incidence::Options {
        session: args.session.clone(),
        since: args.since.clone(),
        exclude_pending: args.exclude_pending,
        config,
    };
    let analysis = match incidence::analyze(&root, ws.model(), &opts) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("archi: {e}");
            return ExitCode::from(1);
        }
    };
    let findings = incidence::filter(&analysis.report.findings, &args.kind, min_severity);
    let pretty = |v: Value| serde_json::to_string_pretty(&v).expect("serializes");
    if args.k_hyper {
        println!("{:.3}", analysis.report.scope.k_hyper);
    } else if args.matrix {
        println!(
            "{}",
            pretty(serde_json::to_value(&analysis.report.matrix).expect("serializes"))
        );
    } else if args.findings {
        println!(
            "{}",
            pretty(serde_json::to_value(&findings).expect("serializes"))
        );
    } else if args.json {
        println!(
            "{}",
            pretty(incidence::to_json(&analysis, args.no_matrix, &findings))
        );
    } else {
        print!(
            "{}",
            incidence::render_human(&analysis, args.no_matrix, &findings)
        );
    }
    ExitCode::SUCCESS
}

fn run_nkp(args: &Args) -> ExitCode {
    if [args.regime, args.hotspots, args.corridors]
        .iter()
        .filter(|f| **f)
        .count()
        > 1
    {
        return usage_err("--regime, --hotspots and --corridors are mutually exclusive");
    }
    if args.top && args.scope.is_some() {
        return usage_err("--top and --scope are mutually exclusive");
    }
    let ws = match analysis_workspace(args) {
        Ok(ws) => ws,
        Err(code) => return code,
    };

    let mut config = NkpConfig::default();
    if !args.exclude.is_empty() {
        config.exclude.clear();
        for pat in &args.exclude {
            match ExcludePattern::parse(pat) {
                Ok(p) => config.exclude.push(p),
                Err(e) => return usage_err(&format!("--exclude `{pat}`: {}", e.message)),
            }
        }
    }
    if args.top {
        config.scope = NkpScope::TopLevel;
    }
    if let Some(p) = &args.scope {
        config.scope = NkpScope::Children(p.clone());
    }
    if !args.only.is_empty() {
        config.only_edge_types = Some(args.only.clone());
    }
    if let Some(t) = args.tau_p {
        config.tau_p = t;
    }
    if let Some(t) = args.tau_b {
        config.tau_b = t;
    }
    match args.neutrality.as_deref() {
        None => {}
        Some("degree") => config.neutrality = Neutrality::DegreeDerived,
        Some("uniform") => config.neutrality = Neutrality::Uniform(args.global_p.unwrap_or(0.5)),
        Some(other) => {
            return usage_err(&format!(
                "--neutrality is `degree` or `uniform`, got `{other}`"
            ));
        }
    }

    let report = match ws.model().nkp(&config) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("archi: {e}");
            return ExitCode::from(1);
        }
    };
    let pretty = |v: Value| serde_json::to_string_pretty(&v).expect("serializes");
    if args.regime {
        let regime = serde_json::to_value(report.metrics.regime).expect("serializes");
        println!("{}", regime.as_str().expect("regime is a string"));
    } else if args.hotspots {
        println!(
            "{}",
            pretty(serde_json::to_value(&report.hotspots).expect("serializes"))
        );
    } else if args.corridors {
        println!(
            "{}",
            pretty(serde_json::to_value(&report.neutral_corridors).expect("serializes"))
        );
    } else {
        println!(
            "{}",
            pretty(serde_json::to_value(&report).expect("serializes"))
        );
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(e) => return usage_err(&e),
    };
    match args.verb.as_str() {
        "check" => run_check(&args),
        "build" => run_build(&args),
        "nkp" => run_nkp(&args),
        "incidence" => run_incidence(&args),
        "version" => run_version(&args),
        "link" => run_link(&args),
        "plan" => run_plan(&args),
        "read" => run_read(&args),
        "query" => run_query(&args),
        other => usage_err(&format!("unknown command `{other}`")),
    }
}
