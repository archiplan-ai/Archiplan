//! `archi` — a thin runner for statement batches (`requirements/cli.md`).
//!
//! ```text
//! archi exec [--dry-run] [--expect-revision <N>] [--model <file>] [--json] [<batch.json> | -]
//! archi ports <Path> [--in <View,...>] [--model <file>] [--json]
//! archi check [--in <View,...>] [--model <file>] [--json]
//! archi dump [--in <View,...>] [--model <file>] [--json]
//! ```
//!
//! The model persists as `{ "revision": N, "statements": [<dump>] }` in the
//! model file (default `archi.json`); how a model is located is provisional
//! until the distribution requirements land.

use std::fs;
use std::io::Read;
use std::process::ExitCode;

use modeling_lang::{Outcome, Response, Statement, Workspace, parse_statement};
use serde_json::{Value, json};

const USAGE: &str = "usage:
  archi exec [--dry-run] [--expect-revision <N>] [--model <file>] [--json] [<batch.json> | -]
  archi ports <Path> [--in <View,...>] [--model <file>] [--json]
  archi check [--in <View,...>] [--model <file>] [--json]
  archi dump [--in <View,...>] [--model <file>] [--json]";

struct Args {
    verb: String,
    positional: Vec<String>,
    model_file: String,
    json: bool,
    dry_run: bool,
    expect_revision: Option<u64>,
    in_views: Vec<String>,
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
        in_views: Vec::new(),
    };
    let mut it = argv.iter().peekable();
    let Some(verb) = it.next() else {
        return Err("missing command".into());
    };
    args.verb = verb.clone();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => args.json = true,
            "--dry-run" => args.dry_run = true,
            "--model" => {
                args.model_file = it.next().ok_or("--model needs a file")?.clone();
            }
            "--expect-revision" => {
                let v = it.next().ok_or("--expect-revision needs a number")?;
                args.expect_revision =
                    Some(v.parse().map_err(|_| "--expect-revision needs a number")?);
            }
            "--in" => {
                let v = it.next().ok_or("--in needs a view list")?;
                args.in_views = v.split(',').map(|s| s.trim().to_string()).collect();
            }
            other if other.starts_with("--") => return Err(format!("unknown flag `{other}`")),
            other => args.positional.push(other.to_string()),
        }
    }
    Ok(args)
}

fn load_workspace(path: &str) -> Result<Workspace, String> {
    let Ok(raw) = fs::read_to_string(path) else {
        return Ok(Workspace::new());
    };
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
    Workspace::restore(revision, &statements)
        .map_err(|e| format!("model file `{path}` does not replay: {e}"))
}

fn save_workspace(path: &str, ws: &Workspace) -> Result<(), String> {
    let dump: Vec<Value> = ws.model().dump().iter().map(Statement::to_value).collect();
    let out = json!({ "revision": ws.revision(), "statements": dump });
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
                    Outcome::Statements { statements } => {
                        println!("{pseudo}");
                        if statements.is_empty() {
                            println!("  (empty)");
                        }
                        for s in statements {
                            println!("  {}", s.pseudo());
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

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(e) => return usage_err(&e),
    };

    let batch: Vec<Value> = match args.verb.as_str() {
        "exec" => match read_batch(&args) {
            Ok(b) => b,
            Err(e) => return usage_err(&e),
        },
        "ports" => {
            let Some(path) = args.positional.first() else {
                return usage_err("ports needs a node path");
            };
            let mut stmt = json!({ "stmt": "ports", "node": path });
            if !args.in_views.is_empty() {
                stmt["in"] = json!(args.in_views);
            }
            vec![stmt]
        }
        "check" | "dump" => {
            let mut stmt = json!({ "stmt": args.verb });
            if !args.in_views.is_empty() {
                stmt["in"] = json!(args.in_views);
            }
            vec![stmt]
        }
        other => return usage_err(&format!("unknown command `{other}`")),
    };

    let mut ws = match load_workspace(&args.model_file) {
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
    let code = run(&response, &args, &batch);
    if response.status == "ok"
        && ws.revision() != before
        && let Err(e) = save_workspace(&args.model_file, &ws)
    {
        eprintln!("archi: {e}");
        return ExitCode::from(2);
    }
    code
}
