//! `archi` — a thin runner for statement batches (`requirements/cli.md`)
//! plus the NKP landscape analysis (`requirements/scoring/nkp.md`).
//!
//! ```text
//! archi exec [--dry-run] [--expect-revision <N>] [--model <file>] [--preset <file>] [--json] [<batch.json> | -]
//! archi nkp  [--model <file>] [--regime | --hotspots | --corridors] [--top | --scope <path>]
//!            [--exclude '<src> <rel> <dst>']... [--only <edge-type>]...
//!            [--tau-p <f>] [--tau-b <f>] [--neutrality degree|uniform] [--global-p <f>]
//! ```
//!
//! The model persists as `{ "revision": N, "preset": { "name", "statements" },
//! "statements": [<dump>] }` in the model file (default `archi.json`); how a
//! model is located is provisional until the distribution requirements land.
//! The preset is pinned at model creation: `--preset <file>`, else an
//! `ontology.json` next to the model file, else the built-in default
//! ontology. Files from before presets replay on the core preset.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use modeling_lang::{
    EdgeKind, ExcludePattern, GraphEdge, Neutrality, NkpConfig, NkpScope, Outcome, Preset,
    Response, Statement, Workspace, parse_statement,
};
use serde_json::{Value, json};

const USAGE: &str = "usage:
  archi exec [--dry-run] [--expect-revision <N>] [--model <file>] [--preset <file>] [--json] [<batch.json> | -]
  archi nkp  [--model <file>] [--regime | --hotspots | --corridors] [--top | --scope <path>]
             [--exclude '<src> <rel> <dst>']... [--only <edge-type>]...
             [--tau-p <f>] [--tau-b <f>] [--neutrality degree|uniform] [--global-p <f>]";

struct Args {
    verb: String,
    positional: Vec<String>,
    model_file: String,
    json: bool,
    dry_run: bool,
    expect_revision: Option<u64>,
    preset: Option<String>,
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
        positional: Vec::new(),
        model_file: "archi.json".to_string(),
        json: false,
        dry_run: false,
        expect_revision: None,
        preset: None,
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
            "--dry-run" => args.dry_run = true,
            "--regime" => args.regime = true,
            "--hotspots" => args.hotspots = true,
            "--corridors" => args.corridors = true,
            "--top" => args.top = true,
            "--model" => args.model_file = value(&mut it, "--model")?,
            "--preset" => args.preset = Some(value(&mut it, "--preset")?),
            "--scope" => args.scope = Some(value(&mut it, "--scope")?),
            "--exclude" => args.exclude.push(value(&mut it, "--exclude")?),
            "--only" => args.only.push(value(&mut it, "--only")?),
            "--neutrality" => args.neutrality = Some(value(&mut it, "--neutrality")?),
            "--tau-p" => args.tau_p = Some(float(value(&mut it, "--tau-p")?, "--tau-p")?),
            "--tau-b" => args.tau_b = Some(float(value(&mut it, "--tau-b")?, "--tau-b")?),
            "--global-p" => {
                args.global_p = Some(float(value(&mut it, "--global-p")?, "--global-p")?)
            }
            "--expect-revision" => {
                let v = value(&mut it, "--expect-revision")?;
                args.expect_revision =
                    Some(v.parse().map_err(|_| "--expect-revision needs a number")?);
            }
            other if other.starts_with("--") => return Err(format!("unknown flag `{other}`")),
            other => args.positional.push(other.to_string()),
        }
    }
    Ok(args)
}

/// The preset for a model file that does not exist yet: `--preset <file>`,
/// else an `ontology.json` next to the model file, else the built-in default
/// ontology.
fn new_model_preset(model_path: &str, flag: Option<&str>) -> Result<Preset, String> {
    let file = match flag {
        Some(f) => Some(PathBuf::from(f)),
        None => {
            let sibling = Path::new(model_path)
                .parent()
                .unwrap_or(Path::new(""))
                .join("ontology.json");
            sibling.exists().then_some(sibling)
        }
    };
    let Some(file) = file else {
        return Ok(Preset::default_ontology());
    };
    let raw = fs::read_to_string(&file)
        .map_err(|e| format!("cannot read preset `{}`: {e}", file.display()))?;
    let v: Value = serde_json::from_str(&raw)
        .map_err(|e| format!("preset `{}` is not JSON: {e}", file.display()))?;
    let name = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("preset");
    Preset::from_value(name, &v).map_err(|e| format!("preset `{}`: {e}", file.display()))
}

fn load_workspace(path: &str, preset_flag: Option<&str>) -> Result<Workspace, String> {
    let Ok(raw) = fs::read_to_string(path) else {
        let preset = new_model_preset(path, preset_flag)?;
        return Workspace::with_preset(&preset).map_err(|e| format!("preset does not load: {e}"));
    };
    if preset_flag.is_some() {
        return Err(format!(
            "`{path}` already exists; its preset was pinned at creation — `--preset` applies to new models only"
        ));
    }
    let v: Value =
        serde_json::from_str(&raw).map_err(|e| format!("model file `{path}` is not JSON: {e}"))?;
    let revision = v
        .get("revision")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("model file `{path}` has no numeric `revision`"))?;
    let raw_stmts = v
        .get("statements")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("model file `{path}` has no `statements` array"))?;
    let mut statements = Vec::with_capacity(raw_stmts.len());
    for s in raw_stmts {
        statements
            .push(parse_statement(s).map_err(|e| format!("model file `{path}` is corrupt: {e}"))?);
    }
    // Files from before presets carry no pin and replay on the core preset.
    let preset = match v.get("preset") {
        None => Preset::core(),
        Some(p) => {
            let name = p
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("model file `{path}` preset has no `name`"))?;
            let stmts = p
                .get("statements")
                .ok_or_else(|| format!("model file `{path}` preset has no `statements`"))?;
            Preset::from_value(name, stmts)
                .map_err(|e| format!("model file `{path}` preset does not load: {e}"))?
        }
    };
    Workspace::restore(&preset, revision, &statements)
        .map_err(|e| format!("model file `{path}` does not replay: {e}"))
}

fn save_workspace(path: &str, ws: &Workspace) -> Result<(), String> {
    let dump: Vec<Value> = ws.model().dump().iter().map(Statement::to_value).collect();
    let preset_stmts: Vec<Value> = ws
        .preset()
        .statements()
        .iter()
        .map(Statement::to_value)
        .collect();
    let out = json!({
        "revision": ws.revision(),
        "preset": { "name": ws.preset().name(), "statements": preset_stmts },
        "statements": dump,
    });
    fs::write(
        path,
        serde_json::to_string_pretty(&out).expect("serializes"),
    )
    .map_err(|e| format!("cannot write `{path}`: {e}"))
}

fn read_batch(args: &Args) -> Result<Vec<Value>, String> {
    let source = match args.positional.first().map(String::as_str) {
        None | Some("-") => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("cannot read stdin: {e}"))?;
            buf
        }
        Some(file) => fs::read_to_string(file).map_err(|e| format!("cannot read `{file}`: {e}"))?,
    };
    let v: Value = serde_json::from_str(&source).map_err(|e| format!("batch is not JSON: {e}"))?;
    match v {
        Value::Array(items) => Ok(items),
        _ => Err("a batch is a JSON array of statements".into()),
    }
}

fn pseudo_of_value(v: &Value) -> String {
    match parse_statement(v) {
        Ok(s) => s.pseudo(),
        Err(_) => v.to_string(),
    }
}

/// A graph edge as one human line, close to the spec's pseudo-syntax.
fn edge_line(e: &GraphEdge) -> String {
    let views = if e.views.is_empty() {
        String::new()
    } else {
        format!(" in {}", e.views.join(", "))
    };
    let type_name = e.type_name.as_deref().unwrap_or("");
    let source_port = e.source_port.as_deref().unwrap_or("");
    let target_port = e.target_port.as_deref().unwrap_or("");
    match e.kind {
        EdgeKind::Relation => format!("{} {type_name} {}{views}", e.source, e.target),
        EdgeKind::Connection => {
            let carrier = e
                .carrier
                .as_ref()
                .map(|c| format!("({c})"))
                .unwrap_or_default();
            format!(
                "{}({source_port}) {type_name}{carrier} {}({target_port}){views}",
                e.source, e.target
            )
        }
        EdgeKind::Application => {
            let route = e
                .route
                .as_ref()
                .map(|r| format!("({r})"))
                .unwrap_or_default();
            format!(
                "{}.{source_port}{route} = {}({target_port}){views}",
                e.source, e.target
            )
        }
    }
}

fn print_human(response: &Response, statements: &[Value]) {
    match (&response.results, &response.error) {
        (Some(results), _) => {
            for (i, outcome) in results.iter().enumerate() {
                let pseudo = statements.get(i).map(pseudo_of_value).unwrap_or_default();
                match outcome {
                    Outcome::Applied { cascade } => {
                        println!("applied   {pseudo}");
                        if let Some(cascade) = cascade {
                            println!("          cascade:");
                            for s in cascade {
                                println!("            {}", s.pseudo());
                            }
                        }
                    }
                    Outcome::Noop => println!("noop      {pseudo}"),
                    Outcome::Graph { nodes, edges } => {
                        println!("{pseudo}");
                        if nodes.is_empty() && edges.is_empty() {
                            println!("  (empty)");
                        }
                        for n in nodes {
                            let types = if n.types.is_empty() {
                                String::new()
                            } else {
                                format!(" : {}", n.types.join(", "))
                            };
                            println!("  node {}{types}", n.id);
                        }
                        for e in edges {
                            println!("  edge {}", edge_line(e));
                        }
                    }
                    Outcome::Findings { findings } => {
                        println!("{pseudo}");
                        if findings.is_empty() {
                            println!("  no findings");
                        }
                        for f in findings {
                            println!("  {f}");
                        }
                    }
                }
            }
            println!("revision {}", response.revision);
        }
        (None, Some(err)) => {
            match err.index {
                Some(i) => eprintln!(
                    "error at statement {i} — {}: {}",
                    err.error.code, err.error.message
                ),
                None => eprintln!("error — {}: {}", err.error.code, err.error.message),
            }
            if let Some(subject) = &err.error.subject {
                eprintln!("  subject: {}", pseudo_of_value(subject));
            }
            if let Some(hint) = &err.error.hint {
                eprintln!("  hint:    {}", pseudo_of_value(hint));
            }
        }
        _ => {}
    }
}

fn run(response: &Response, args: &Args, batch: &[Value]) -> ExitCode {
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(response).expect("serializes")
        );
    } else {
        print_human(response, batch);
    }
    if response.status == "ok" {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_exec(args: &Args) -> ExitCode {
    let batch = match read_batch(args) {
        Ok(b) => b,
        Err(e) => return usage_err(&e),
    };
    let existed = Path::new(&args.model_file).exists();
    let mut ws = match load_workspace(&args.model_file, args.preset.as_deref()) {
        Ok(ws) => ws,
        Err(e) => return usage_err(&e),
    };
    let before = ws.revision();

    let mut request = json!({ "statements": batch });
    if args.dry_run {
        request["dry_run"] = json!(true);
    }
    if let Some(n) = args.expect_revision {
        request["expect_revision"] = json!(n);
    }

    let response = ws.handle(&request);
    let code = run(&response, args, &batch);
    // A fresh model file is written even for a no-op batch: creation is what
    // pins the preset.
    if response.status == "ok"
        && (ws.revision() != before || (!existed && !args.dry_run))
        && let Err(e) = save_workspace(&args.model_file, &ws)
    {
        eprintln!("archi: {e}");
        return ExitCode::from(2);
    }
    code
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
    let ws = match load_workspace(&args.model_file, args.preset.as_deref()) {
        Ok(ws) => ws,
        Err(e) => return usage_err(&e),
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
        "exec" => run_exec(&args),
        "nkp" => run_nkp(&args),
        other => usage_err(&format!("unknown command `{other}`")),
    }
}
