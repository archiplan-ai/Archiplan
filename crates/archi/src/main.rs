//! `archi` — a thin runner for the `.arch` source-format compiler
//! (`requirements/cli.md`, `requirements/modeling-lang/source-format.md`) and
//! the NKP landscape analysis (`requirements/scoring/nkp.md`).
//!
//! ```text
//! archi check [--project <dir>] [--json]
//! archi build [--project <dir>] [--emit-batch <file|->]
//! archi nkp   [--project <dir>] [--regime | --hotspots | --corridors] [--top | --scope <path>]
//!             [--exclude '<src> <rel> <dst>']... [--only <edge-type>]...
//!             [--tau-p <f>] [--tau-b <f>] [--neutrality degree|uniform] [--global-p <f>]
//! ```
//!
//! Every verb locates its project by precedence: `--project`, then the
//! nearest `archi.toml` upward from the working directory. A project of
//! `.arch` files is compiled fresh each run — the source is the model, and
//! the only source of truth: the CLI offers no JSON editing of the model;
//! mutation is a text edit and a recompile.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use modeling_lang::source::{Compiled, compile_project, find_project_root};
use modeling_lang::{
    ExcludePattern, Finding, Neutrality, NkpConfig, NkpScope, Statement, Workspace,
};
use serde_json::{Value, json};

const USAGE: &str = "usage:
  archi check [--project <dir>] [--json]
  archi build [--project <dir>] [--emit-batch <file|->]
  archi nkp   [--project <dir>] [--regime | --hotspots | --corridors] [--top | --scope <path>]
              [--exclude '<src> <rel> <dst>']... [--only <edge-type>]...
              [--tau-p <f>] [--tau-b <f>] [--neutrality degree|uniform] [--global-p <f>]";

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
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => args.json = true,
            "--regime" => args.regime = true,
            "--hotspots" => args.hotspots = true,
            "--corridors" => args.corridors = true,
            "--top" => args.top = true,
            "--project" => args.project = Some(value(&mut it, "--project")?),
            "--emit-batch" => args.emit_batch = Some(value(&mut it, "--emit-batch")?),
            "--scope" => args.scope = Some(value(&mut it, "--scope")?),
            "--exclude" => args.exclude.push(value(&mut it, "--exclude")?),
            "--only" => args.only.push(value(&mut it, "--only")?),
            "--neutrality" => args.neutrality = Some(value(&mut it, "--neutrality")?),
            "--tau-p" => args.tau_p = Some(float(value(&mut it, "--tau-p")?, "--tau-p")?),
            "--tau-b" => args.tau_b = Some(float(value(&mut it, "--tau-b")?, "--tau-b")?),
            "--global-p" => {
                args.global_p = Some(float(value(&mut it, "--global-p")?, "--global-p")?)
            }
            other if other.starts_with("--") => return Err(format!("unknown flag `{other}`")),
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
    let ws = match analysis_workspace(args) {
        Ok(w) => w,
        Err(code) => return code,
    };
    let findings: Vec<Finding> = ws.model().check();
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({ "status": "ok", "findings": findings }))
                .expect("serializes")
        );
    } else if findings.is_empty() {
        println!("no findings");
    } else {
        for f in &findings {
            println!("{f}");
        }
    }
    ExitCode::SUCCESS
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
        other => usage_err(&format!("unknown command `{other}`")),
    }
}
