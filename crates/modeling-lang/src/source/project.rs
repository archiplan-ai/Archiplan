//! Project layout: the `archi.toml` manifest, module discovery and preset
//! selection.
//!
//! A project is a directory with an `archi.toml` at its root and `.arch`
//! modules under the source directory (default `archi/src/`, keeping the
//! model out of the way of a host project's own `src/`). One file is one
//! module; its module path is the dotted path relative to the source dir —
//! `archi/src/auth/service.arch` is `auth.service`. Discovery is a sorted
//! walk, so module order — and with it every downstream ordering — is
//! independent of filesystem iteration.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::preset::Preset;

use super::span::Diagnostic;

/// The parsed `archi.toml`.
#[derive(Clone, Debug)]
pub(crate) struct Manifest {
    /// Required by the manifest format; not consumed by the compiler yet.
    #[allow(dead_code)]
    pub name: String,
    /// Source directory, relative to the project root.
    pub src: String,
    /// Preset selection: `core`, `default`, or a relative path to a JSON
    /// preset file.
    pub preset: String,
    /// Declared member repositories, in declaration order.
    pub repos: Vec<RepoDecl>,
    /// Branches where mutating verbs refuse, consumed by `archi`; empty
    /// means no protection.
    pub protected: Vec<String>,
}

/// One declared member repository: the name is the identity refs and journal
/// events carry, the url is provenance for humans and CI, the path a layout
/// convention relative to the project root.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepoDecl {
    /// The stable identity: refs (`name//file#symbol`) and journal events
    /// carry it; renaming it is a journal migration.
    pub name: String,
    /// Where the repository lives remotely — provenance for humans and CI;
    /// archi never fetches it and it keys nothing.
    pub url: Option<String>,
    /// The committed checkout convention, relative to the project root;
    /// the machine-local overlay overrides it.
    pub path: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestFile {
    project: ProjectSection,
    /// Settings for the link layer's tree scans, consumed by `archi` —
    /// validated here so a typo inside the section is loud at compile
    /// time instead of a silently ignored setting.
    #[allow(dead_code)]
    audit: Option<AuditSection>,
    /// `[[repo]]` member declarations, consumed by `archi` — validated
    /// here for the same loud-typo reason as `[audit]`.
    repo: Option<Vec<RepoDecl>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectSection {
    name: String,
    src: Option<String>,
    preset: Option<String>,
    /// Protected branches, consumed by `archi` — validated here for the
    /// same loud-typo reason as `[audit]`.
    protected: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuditSection {
    /// Scan-exclusion patterns: `dir/` prefix, `*.ext` suffix, or exact
    /// path. Governs what the scans volunteer, never what links may claim.
    #[allow(dead_code)]
    exclude: Option<Vec<String>>,
}

fn project_err(message: impl Into<String>) -> Diagnostic {
    Diagnostic::project("E_PROJECT", message)
}

/// Locate a project root: `dir` itself or the nearest ancestor holding an
/// `archi.toml`.
pub(crate) fn find_root(dir: &Path) -> Option<PathBuf> {
    let mut cur = Some(dir);
    while let Some(d) = cur {
        if d.join("archi.toml").is_file() {
            return Some(d.to_path_buf());
        }
        cur = d.parent();
    }
    None
}

/// Read and validate `archi.toml` at the project root.
pub(crate) fn read_manifest(root: &Path) -> Result<Manifest, Diagnostic> {
    let path = root.join("archi.toml");
    let text = fs::read_to_string(&path)
        .map_err(|e| project_err(format!("cannot read {}: {e}", path.display())))?;
    let parsed: ManifestFile =
        toml::from_str(&text).map_err(|e| project_err(format!("{}: {e}", path.display())))?;
    let repos = parsed.repo.unwrap_or_default();
    let mut seen = std::collections::BTreeSet::new();
    for r in &repos {
        if !is_member_name(&r.name) {
            return Err(project_err(format!(
                "{}: member name `{}` is not ref-safe — ascii letters, digits, `-` and `_`, starting alphanumeric",
                path.display(),
                r.name
            )));
        }
        if !seen.insert(r.name.as_str()) {
            return Err(project_err(format!(
                "{}: member `{}` declared twice",
                path.display(),
                r.name
            )));
        }
    }
    Ok(Manifest {
        name: parsed.project.name,
        src: parsed.project.src.unwrap_or_else(|| "archi/src".to_string()),
        preset: parsed
            .project
            .preset
            .unwrap_or_else(|| "default".to_string()),
        repos,
        protected: parsed.project.protected.unwrap_or_default(),
    })
}

/// A member name must be able to prefix a `member//file#symbol` ref: ascii
/// letters, digits, `-` and `_`, first char alphanumeric.
fn is_member_name(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphanumeric())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Resolve the manifest's preset choice to a loaded [`Preset`].
pub(crate) fn resolve_preset(root: &Path, manifest: &Manifest) -> Result<Preset, Diagnostic> {
    match manifest.preset.as_str() {
        "core" => Ok(Preset::core()),
        "default" => Ok(Preset::default_ontology()),
        path => {
            let full = root.join(path);
            let text = fs::read_to_string(&full)
                .map_err(|e| project_err(format!("cannot read preset {}: {e}", full.display())))?;
            let value: serde_json::Value = serde_json::from_str(&text)
                .map_err(|e| project_err(format!("preset {}: {e}", full.display())))?;
            let name = full
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "preset".to_string());
            Preset::from_value(&name, &value)
                .map_err(|e| project_err(format!("preset {}: {e}", full.display())))
        }
    }
}

/// One discovered module: its dotted path, display path and text.
#[derive(Clone, Debug)]
pub(crate) struct ModuleSource {
    /// Dotted module path (`auth.service`).
    pub module: String,
    /// Display path relative to the project root (`archi/src/auth/service.arch`).
    pub rel_path: String,
    pub text: String,
}

fn is_module_ident(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Discover every `.arch` module under the source directory, in sorted order.
pub(crate) fn discover_modules(root: &Path, src: &str) -> Result<Vec<ModuleSource>, Diagnostic> {
    let src_dir = root.join(src);
    if !src_dir.is_dir() {
        return Err(project_err(format!(
            "source directory {} does not exist",
            src_dir.display()
        )));
    }
    let mut out = Vec::new();
    walk(root, &src_dir, &mut Vec::new(), &mut out)?;
    out.sort_by(|a, b| a.module.cmp(&b.module));
    Ok(out)
}

fn walk(
    root: &Path,
    dir: &Path,
    prefix: &mut Vec<String>,
    out: &mut Vec<ModuleSource>,
) -> Result<(), Diagnostic> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| project_err(format!("cannot read {}: {e}", dir.display())))?
        .collect::<Result<_, _>>()
        .map_err(|e| project_err(format!("cannot read {}: {e}", dir.display())))?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if file_name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            if !is_module_ident(&file_name) {
                return Err(project_err(format!(
                    "directory `{file_name}` is not a valid module segment ({})",
                    path.display()
                )));
            }
            prefix.push(file_name);
            walk(root, &path, prefix, out)?;
            prefix.pop();
        } else if let Some(stem) = file_name.strip_suffix(".arch") {
            if !is_module_ident(stem) {
                return Err(project_err(format!(
                    "file `{file_name}` is not a valid module name ({})",
                    path.display()
                )));
            }
            let mut segs = prefix.clone();
            segs.push(stem.to_string());
            let text = fs::read_to_string(&path)
                .map_err(|e| project_err(format!("cannot read {}: {e}", path.display())))?;
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            out.push(ModuleSource {
                module: segs.join("."),
                rel_path,
                text,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn manifest_of(text: &str) -> Result<Manifest, Diagnostic> {
        let dir = std::env::temp_dir().join(format!(
            "archi-manifest-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("archi.toml"), text).unwrap();
        let out = read_manifest(&dir);
        fs::remove_dir_all(&dir).unwrap();
        out
    }

    #[test]
    fn manifest_accepts_a_valid_audit_section() {
        let m = manifest_of("[project]\nname = \"t\"\n\n[audit]\nexclude = [\"*.md\", \"notes/\"]\n")
            .unwrap();
        assert_eq!(m.name, "t");
    }

    #[test]
    fn a_typo_inside_audit_is_loud() {
        let e = manifest_of("[project]\nname = \"t\"\n\n[audit]\nexclud = [\"*.md\"]\n").unwrap_err();
        assert_eq!(e.code, "E_PROJECT");
        assert!(e.message.contains("exclud"), "{}", e.message);
    }

    #[test]
    fn a_non_list_exclude_is_loud() {
        let e = manifest_of("[project]\nname = \"t\"\n\n[audit]\nexclude = \"*.md\"\n").unwrap_err();
        assert_eq!(e.code, "E_PROJECT");
    }

    #[test]
    fn repo_sections_parse_in_order() {
        let m = manifest_of(
            "[project]\nname = \"t\"\n\n[[repo]]\nname = \"backend\"\npath = \"../backend\"\n\n[[repo]]\nname = \"web-ui\"\nurl = \"git@example.com:acme/web.git\"\n",
        )
        .unwrap();
        assert_eq!(m.repos.len(), 2);
        assert_eq!(m.repos[0].name, "backend");
        assert_eq!(m.repos[0].path.as_deref(), Some("../backend"));
        assert_eq!(m.repos[1].name, "web-ui");
        assert_eq!(m.repos[1].url.as_deref(), Some("git@example.com:acme/web.git"));
        assert_eq!(m.repos[1].path, None);
    }

    #[test]
    fn a_memberless_manifest_yields_the_empty_list() {
        let m = manifest_of("[project]\nname = \"t\"\n").unwrap();
        assert!(m.repos.is_empty());
    }

    #[test]
    fn protected_branches_parse_and_default_empty() {
        let m =
            manifest_of("[project]\nname = \"t\"\nprotected = [\"main\", \"release\"]\n").unwrap();
        assert_eq!(m.protected, vec!["main", "release"]);
        let m = manifest_of("[project]\nname = \"t\"\n").unwrap();
        assert!(m.protected.is_empty());
    }

    #[test]
    fn a_typo_of_protected_is_loud() {
        let e = manifest_of("[project]\nname = \"t\"\nprotectd = [\"main\"]\n").unwrap_err();
        assert_eq!(e.code, "E_PROJECT");
        assert!(e.message.contains("protectd"), "{}", e.message);
    }

    #[test]
    fn a_ref_unsafe_member_name_is_loud() {
        for bad in ["a/b", "a#b", "a b", "", "-lead", "café"] {
            let text = format!("[project]\nname = \"t\"\n\n[[repo]]\nname = \"{bad}\"\n");
            let e = manifest_of(&text).unwrap_err();
            assert_eq!(e.code, "E_PROJECT");
            assert!(e.message.contains("ref-safe"), "{}", e.message);
        }
    }

    #[test]
    fn a_duplicate_member_name_is_loud() {
        let e = manifest_of(
            "[project]\nname = \"t\"\n\n[[repo]]\nname = \"backend\"\n\n[[repo]]\nname = \"backend\"\n",
        )
        .unwrap_err();
        assert!(e.message.contains("declared twice"), "{}", e.message);
    }

    #[test]
    fn a_typo_inside_repo_is_loud() {
        let e = manifest_of("[project]\nname = \"t\"\n\n[[repo]]\nname = \"backend\"\nurll = \"x\"\n")
            .unwrap_err();
        assert_eq!(e.code, "E_PROJECT");
        assert!(e.message.contains("urll"), "{}", e.message);
    }
}
