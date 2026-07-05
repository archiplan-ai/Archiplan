#![allow(dead_code)]

use modeling_lang::{ErrorCode, Finding, LangError, Outcome, Statement, Workspace};
use serde_json::Value;

pub fn ws_with(batch: Value) -> Workspace {
    let mut ws = Workspace::new();
    match ws.execute_values(batch.as_array().expect("batch is an array")) {
        Ok(_) => ws,
        Err(e) => panic!("setup batch failed: {e}"),
    }
}

pub fn outcomes(ws: &mut Workspace, batch: Value) -> Vec<Outcome> {
    ws.execute_values(batch.as_array().expect("batch is an array"))
        .unwrap_or_else(|e| panic!("batch failed: {e}"))
}

pub fn outcome(ws: &mut Workspace, stmt: Value) -> Outcome {
    outcomes(ws, Value::Array(vec![stmt]))
        .pop()
        .expect("one outcome")
}

pub fn err(ws: &mut Workspace, batch: Value) -> (usize, LangError) {
    match ws.execute_values(batch.as_array().expect("batch is an array")) {
        Ok(r) => panic!("expected failure, got {r:?}"),
        Err(b) => (b.index, b.error),
    }
}

pub fn err_code(ws: &mut Workspace, stmt: Value) -> ErrorCode {
    err(ws, Value::Array(vec![stmt])).1.code
}

pub fn statements(ws: &mut Workspace, stmt: Value) -> Vec<Statement> {
    match outcome(ws, stmt) {
        Outcome::Statements { statements } => statements,
        o => panic!("expected statements, got {o:?}"),
    }
}

pub fn pseudo(ws: &mut Workspace, stmt: Value) -> Vec<String> {
    statements(ws, stmt).iter().map(Statement::pseudo).collect()
}

pub fn findings(ws: &mut Workspace, stmt: Value) -> Vec<Finding> {
    match outcome(ws, stmt) {
        Outcome::Findings { findings } => findings,
        o => panic!("expected findings, got {o:?}"),
    }
}

pub fn cascade(ws: &mut Workspace, stmt: Value) -> Vec<String> {
    match outcome(ws, stmt) {
        Outcome::Applied { cascade: Some(c) } => c.iter().map(Statement::pseudo).collect(),
        o => panic!("expected a cascade, got {o:?}"),
    }
}

pub fn is_applied(o: &Outcome) -> bool {
    matches!(o, Outcome::Applied { .. })
}

pub fn is_noop(o: &Outcome) -> bool {
    matches!(o, Outcome::Noop)
}
