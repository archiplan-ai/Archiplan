#![allow(dead_code)]

use modeling_lang::{ErrorCode, Finding, Outcome, Session};

pub fn session_with(src: &str) -> Session {
    let mut s = Session::new();
    if let Err(e) = s.execute(src) {
        panic!("setup batch failed: {e}");
    }
    s
}

pub fn outcomes(s: &mut Session, src: &str) -> Vec<Outcome> {
    match s.execute(src) {
        Ok(results) => results.into_iter().map(|r| r.outcome).collect(),
        Err(e) => panic!("`{src}` failed: {e}"),
    }
}

pub fn outcome(s: &mut Session, src: &str) -> Outcome {
    outcomes(s, src).pop().expect("at least one statement")
}

pub fn err(s: &mut Session, src: &str) -> modeling_lang::LangError {
    match s.execute(src) {
        Ok(results) => panic!(
            "expected `{src}` to fail, got {:?}",
            results.iter().map(|r| &r.outcome).collect::<Vec<_>>()
        ),
        Err(b) => b.error,
    }
}

pub fn err_code(s: &mut Session, src: &str) -> ErrorCode {
    err(s, src).code
}

pub fn statements(s: &mut Session, query: &str) -> Vec<String> {
    match outcome(s, query) {
        Outcome::Statements(lines) => lines,
        o => panic!("expected statements from `{query}`, got {o:?}"),
    }
}

pub fn findings(s: &mut Session, query: &str) -> Vec<Finding> {
    match outcome(s, query) {
        Outcome::Findings(f) => f,
        o => panic!("expected findings from `{query}`, got {o:?}"),
    }
}

pub fn cascade(s: &mut Session, delete: &str) -> Vec<String> {
    match outcome(s, delete) {
        Outcome::Deleted { cascade } => cascade,
        o => panic!("expected a delete from `{delete}`, got {o:?}"),
    }
}

pub fn is_applied(o: &Outcome) -> bool {
    matches!(o, Outcome::Applied)
}

pub fn is_noop(o: &Outcome) -> bool {
    matches!(o, Outcome::Noop)
}
