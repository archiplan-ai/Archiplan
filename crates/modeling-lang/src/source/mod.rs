//! The `.arch` source format: a diffable textual surface language that
//! compiles to the statement layer (`requirements/modeling-lang/source-format.md`).
//!
//! A project of `.arch` files *is* the stored model: [`compile_project`]
//! parses every module under the source root, resolves imports and lexical
//! scopes to absolute paths, lowers everything to one deterministic statement
//! batch and executes it on a fresh [`Workspace`]. Errors — parse, resolution
//! or engine — come back as [`Diagnostic`]s with `file:line:col` locations.

mod ast;
mod lexer;
mod lower;
mod parser;
mod project;
mod resolve;
mod span;

use std::path::Path;

use crate::error::LangError;
use crate::preset::Preset;
use crate::statement::Statement;
use crate::{BatchError, Workspace};

pub use span::{Diagnostic, FileId, SourceMap, Span};

/// Locate a project root: the given directory or the nearest ancestor with
/// an `archi.toml`.
pub fn find_project_root(dir: &Path) -> Option<std::path::PathBuf> {
    project::find_root(dir)
}

/// A successfully compiled project.
pub struct Compiled {
    /// A fresh workspace holding the compiled model.
    pub workspace: Workspace,
    /// The compiled statement batch, in deterministic lowering order.
    pub batch: Vec<Statement>,
    /// The project's sources, for rendering later diagnostics.
    pub map: SourceMap,
    /// Source span of each statement in `batch`, index-aligned.
    pub spans: Vec<Span>,
}

/// A failed compilation: every collected diagnostic plus the sources to
/// render them against.
pub struct CompileFailure {
    /// The project's sources.
    pub map: SourceMap,
    /// What went wrong, in source order where meaningful.
    pub diagnostics: Vec<Diagnostic>,
}

impl CompileFailure {
    /// All diagnostics rendered as `file:line:col: CODE: message` lines.
    pub fn render(&self) -> String {
        self.diagnostics
            .iter()
            .map(|d| d.render(&self.map))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Compile the project rooted at `root` (the directory holding `archi.toml`).
pub fn compile_project(root: &Path) -> Result<Compiled, CompileFailure> {
    let fail_project = |d: Diagnostic| CompileFailure {
        map: SourceMap::new(),
        diagnostics: vec![d],
    };
    let manifest = project::read_manifest(root).map_err(fail_project)?;
    let preset = project::resolve_preset(root, &manifest).map_err(fail_project)?;
    let sources = project::discover_modules(root, &manifest.src).map_err(fail_project)?;
    compile_modules(&preset, sources)
}

/// Compile in-memory sources — `(module path, text)` pairs — against a
/// preset. What `compile_project` does after project discovery; meant for
/// tests and embedders.
pub fn compile_sources(
    preset: &Preset,
    sources: &[(&str, &str)],
) -> Result<Compiled, CompileFailure> {
    let modules = sources
        .iter()
        .map(|(module, text)| project::ModuleSource {
            module: module.to_string(),
            rel_path: format!("src/{}.arch", module.replace('.', "/")),
            text: text.to_string(),
        })
        .collect();
    compile_modules(preset, modules)
}

fn compile_modules(
    preset: &Preset,
    mut sources: Vec<project::ModuleSource>,
) -> Result<Compiled, CompileFailure> {
    sources.sort_by(|a, b| a.module.cmp(&b.module));
    let mut map = SourceMap::new();
    let mut diagnostics = Vec::new();
    let mut modules = Vec::new();
    for src in &sources {
        let file = map.add_file(src.rel_path.clone(), src.text.clone());
        match parser::parse(file, &src.text) {
            Ok(ast) => modules.push(resolve::ModuleAst {
                module: src.module.clone(),
                ast,
            }),
            Err(d) => diagnostics.push(d),
        }
    }
    if !diagnostics.is_empty() {
        return Err(CompileFailure { map, diagnostics });
    }

    let mut workspace = Workspace::with_preset(preset).map_err(|e| CompileFailure {
        map: SourceMap::new(),
        diagnostics: vec![Diagnostic::project(e.code.as_str(), lang_error_message(&e))],
    })?;
    let preset_info = resolve::PresetInfo::from_workspace(&workspace);

    let resolution = match resolve::resolve(&modules, &preset_info) {
        Ok(r) => r,
        Err(diagnostics) => return Err(CompileFailure { map, diagnostics }),
    };
    let lowered = match lower::lower(&resolution) {
        Ok(l) => l,
        Err(diagnostics) => return Err(CompileFailure { map, diagnostics }),
    };

    if let Err(BatchError { index, error }) = workspace.execute(&lowered.batch) {
        let d = Diagnostic::new(
            error.code.as_str(),
            lang_error_message(&error),
            lowered.spans[index],
        );
        return Err(CompileFailure {
            map,
            diagnostics: vec![d],
        });
    }

    Ok(Compiled {
        workspace,
        batch: lowered.batch,
        map,
        spans: lowered.spans,
    })
}

/// The error message with its violated-constraint context, span-friendly.
fn lang_error_message(e: &LangError) -> String {
    let mut msg = e.message.clone();
    if let Some(exp) = &e.expected {
        msg.push_str(&format!("; expected {exp}"));
    }
    if let Some(act) = &e.actual {
        msg.push_str(&format!("; actual {act}"));
    }
    msg
}
