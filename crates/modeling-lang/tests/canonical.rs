//! The canonical render (`archi/requirements/self-hosting/versions-mint-on-meaning.md`):
//! `render_source` is byte-for-byte deterministic and a fixed point under
//! recompilation, and `scope_sources` slices it per root scope for
//! scope-version hashing.

use modeling_lang::Preset;
use modeling_lang::source::{Compiled, compile_project, compile_sources};
use std::path::Path;

fn compile_fixture() -> Compiled {
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/auth"));
    match compile_project(root) {
        Ok(c) => c,
        Err(f) => panic!("fixture failed to compile:\n{}", f.render()),
    }
}

#[test]
fn render_source_is_deterministic_and_a_fixed_point() {
    let first = compile_fixture().workspace.model().render_source();
    let again = compile_fixture().workspace.model().render_source();
    assert_eq!(first, again, "identical sources must render identically");

    // Compiling the render as a single module against the same preset must
    // reproduce the text byte for byte — versioning's hash contract.
    let recompiled = match compile_sources(&Preset::default_ontology(), &[("model", &first)]) {
        Ok(c) => c,
        Err(f) => panic!(
            "canonical render is not valid source:\n{}\n--- render ---\n{first}",
            f.render()
        ),
    };
    let second = recompiled.workspace.model().render_source();
    assert_eq!(first, second, "render → compile → render is a fixed point");
}

#[test]
fn scope_sources_slice_the_render_per_root() {
    let compiled = compile_fixture();
    let model = compiled.workspace.model();
    let scopes = model.scope_sources();

    // User roots only, in name order — preset ontology roots are omitted.
    let paths: Vec<&str> = scopes.iter().map(|s| s.path.as_str()).collect();
    assert_eq!(
        paths,
        ["AuthResponse", "AuthService", "CredHash", "LoginForm", "UI"]
    );

    // Every fragment line comes from the canonical render.
    let render = model.render_source();
    for s in &scopes {
        for line in s.full.lines().chain(s.interface.lines()) {
            assert!(render.contains(line), "fragment line not in render: {line}");
        }
    }

    let auth = scopes.iter().find(|s| s.path == "AuthService").unwrap();
    // Full: the subtree's defines plus fully-internal edges and delegations.
    assert!(auth.full.contains("def node AuthService:"));
    assert!(auth.full.contains("def node AuthService.Storage:"));
    assert!(auth.full.contains("def node AuthService.LoginHandler:"));
    assert!(auth.full.contains("store(CredHash)"), "{}", auth.full);
    assert!(
        auth.full
            .contains("AuthService.handle_login = LoginHandler.handle")
    );
    // Boundary edges stay out of full…
    assert!(!auth.full.contains("Service type_of AuthService"));
    assert!(!auth.full.contains("UI.login"));
    // …and land in interface, together with the node's own define.
    assert!(auth.interface.contains("def node AuthService:"));
    assert!(auth.interface.contains("port handle_login"));
    assert!(auth.interface.contains("Service type_of AuthService"));
    assert!(
        auth.interface.contains("UI.login login"),
        "{}",
        auth.interface
    );
    // Internals stay out of interface.
    assert!(!auth.interface.contains("Storage"));

    // A portless data root: its define, and its classification as boundary.
    let form = scopes.iter().find(|s| s.path == "LoginForm").unwrap();
    assert_eq!(form.full, "def node LoginForm\n");
    assert!(form.interface.contains("def node LoginForm"));
    assert!(form.interface.contains("Data type_of LoginForm"));
    // The login conn carries LoginForm, but a carrier is metadata, not an
    // attachment — the edge belongs to UI and AuthService, not here.
    assert!(!form.interface.contains("UI.login"));

    // An internals-only change story needs the two fragments to differ.
    let ui = scopes.iter().find(|s| s.path == "UI").unwrap();
    assert!(ui.interface.contains("UI.login login"));
}
