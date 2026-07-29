//! Self-update: `archi check-update` asks the release server where this
//! binary stands; `archi update` swaps the binary for the server's latest.
//!
//! The server contract is the installer's (`release/install.sh`): `GET
//! <BASE>/version` answers `{"server":"…","latest":"x.y.z"}`, and `GET
//! <BASE>/download/archi-<version>-<platform>.tar.gz` serves a tarball
//! unpacking to `archi-<version>-<platform>/archi`. BASE defaults to
//! `https://api.archiplan.ai`; `ARCHI_BASE_URL` points anywhere else —
//! tests ride `file://` fixtures, which curl serves like any other URL.
//!
//! Transport is system plumbing, the same doctrine as git in
//! `worktrees.rs`: `curl -fsSL` fetches, `tar -xzf` unpacks, and refusals
//! carry the tool's own stderr verbatim plus the continuation. The swap
//! itself is one `rename` onto the resolved running binary — atomic, so
//! the new binary lands whole or not at all: every earlier failure (torn
//! download, bad unpack, refused rename) leaves the standing binary
//! untouched. Symlinks resolve to their target first — the link stays a
//! link; the file behind it is what changes. The server is the truth of
//! latest in either direction: a lower number is a rollback, and `update`
//! follows it.

use std::fs;
use std::path::Path;
use std::process::Command;

/// The release server every request rides unless `ARCHI_BASE_URL` says
/// otherwise.
const DEFAULT_BASE: &str = "https://api.archiplan.ai";

/// This binary's own version — the baseline every comparison starts from.
const CURRENT: &str = env!("CARGO_PKG_VERSION");

/// The continuation every server-side refusal names.
const CONTINUATION: &str = "check the network, or point ARCHI_BASE_URL at a release server";

/// The base URL, `ARCHI_BASE_URL` over the default, trailing slash trimmed.
fn base() -> String {
    match std::env::var("ARCHI_BASE_URL") {
        Ok(v) if !v.trim().is_empty() => v.trim().trim_end_matches('/').to_string(),
        _ => DEFAULT_BASE.to_string(),
    }
}

/// The release platform this binary was built for — the compiled-in mirror
/// of `release/install.sh`'s uname map. `Err` is the refusal for platforms
/// the release server does not build: Windows rides its own installer,
/// anything else names itself.
fn platform() -> Result<&'static str, String> {
    if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        return Ok("linux-x64");
    }
    if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        return Ok("linux-arm64");
    }
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        return Ok("macos-arm64");
    }
    if cfg!(target_os = "windows") {
        return Err(
            "self-update on Windows rides the installer: `irm https://api.archiplan.ai/install.ps1 | iex`"
                .into(),
        );
    }
    Err(format!(
        "no release build for {}-{} — supported: linux x86_64/aarch64, macos aarch64 (Apple Silicon)",
        std::env::consts::OS,
        std::env::consts::ARCH
    ))
}

/// `curl -fsSL [extra] <url>`, body on stdout. Loud like a git mutation:
/// `Err` names the URL and carries curl's own stderr verbatim.
fn curl(url: &str, extra: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("curl")
        .arg("-fsSL")
        .args(extra)
        .arg(url)
        .output()
        .map_err(|e| format!("curl {url}: {e}"))?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(format!("curl {url}: {}", String::from_utf8_lossy(&out.stderr).trim()))
    }
}

/// `x.y.z` as a numeric triple — versions order as numbers, so
/// `0.1.9 < 0.1.10`, which string order gets wrong.
fn triple(v: &str) -> Result<(u64, u64, u64), String> {
    let parts: Vec<u64> = v
        .trim()
        .split('.')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(|_| format!("`{v}` is not a version triple (x.y.z)"))?;
    match parts.as_slice() {
        [a, b, c] => Ok((*a, *b, *c)),
        _ => Err(format!("`{v}` is not a version triple (x.y.z)")),
    }
}

/// Ask the server its latest: `GET <base>/version`, the `latest` field of
/// the JSON envelope, validated as a triple. Every refusal names the cause
/// and the continuation.
fn latest(base: &str) -> Result<(String, (u64, u64, u64)), String> {
    let url = format!("{base}/version");
    let body = curl(&url, &[])
        .map_err(|e| format!("the release server gave no version — {e}; {CONTINUATION}"))?;
    let v: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| format!("the answer at {url} is not the version JSON ({e}); {CONTINUATION}"))?;
    let Some(latest) = v.get("latest").and_then(|l| l.as_str()) else {
        return Err(format!("the answer at {url} carries no `latest` version; {CONTINUATION}"));
    };
    let t = triple(latest).map_err(|e| format!("the server's latest is malformed: {e}; {CONTINUATION}"))?;
    Ok((latest.to_string(), t))
}

/// `archi check-update` — one line naming where this binary stands against
/// the server's latest. Never touches the binary.
pub fn check() -> Result<String, String> {
    let (latest_s, latest_t) = latest(&base())?;
    let current_t = triple(CURRENT).map_err(|e| format!("this binary's own version is broken: {e}"))?;
    Ok(match latest_t.cmp(&current_t) {
        std::cmp::Ordering::Equal => format!("archi {CURRENT} — up to date"),
        std::cmp::Ordering::Greater => {
            format!("archi {CURRENT} — newer {latest_s} available: `archi update`")
        }
        std::cmp::Ordering::Less => format!(
            "archi {CURRENT} — the server offers {latest_s} (a rollback): `archi update` follows it"
        ),
    })
}

/// A scratch dir under the system temp for the download and unpack,
/// removed on drop — a torn attempt never leaves litter behind.
struct Scratch(std::path::PathBuf);

impl Scratch {
    fn new() -> Result<Self, String> {
        let dir = std::env::temp_dir().join(format!(
            "archi-update-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir)
            .map_err(|e| format!("cannot make a scratch dir at {}: {e}", dir.display()))?;
        Ok(Scratch(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Mark a staged binary executable (755) — the rename is the atom, so the
/// file must arrive already runnable.
fn mark_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("cannot mark the new binary executable: {e}"))?;
    }
    Ok(())
}

/// The atom: rename the staged binary onto the resolved target — on one
/// filesystem, a single `rename`. Across filesystems (EXDEV, raw os error
/// 18) the stage first copies into the target's own directory, so the
/// final rename is same-device again; a failed fallback tears its staged
/// copy off. Either way a refusal leaves the standing binary untouched.
fn swap(staged: &Path, target: &Path) -> Result<(), String> {
    match fs::rename(staged, target) {
        Ok(()) => Ok(()),
        Err(e) if e.raw_os_error() == Some(18) => {
            let dir = target
                .parent()
                .ok_or_else(|| format!("the target {} has no parent directory", target.display()))?;
            let tmp = dir.join(format!(".archi-update-{}", std::process::id()));
            fs::copy(staged, &tmp).map_err(|e| {
                format!("the swap failed (staging into {}: {e}) — the standing binary is untouched", dir.display())
            })?;
            if let Err(e) = mark_executable(&tmp) {
                let _ = fs::remove_file(&tmp);
                return Err(format!("the swap failed ({e}) — the standing binary is untouched"));
            }
            fs::rename(&tmp, target).map_err(|e| {
                let _ = fs::remove_file(&tmp);
                format!("the swap failed (rename onto {}: {e}) — the standing binary is untouched", target.display())
            })
        }
        Err(e) => Err(format!(
            "the swap failed (rename onto {}: {e}) — the standing binary is untouched",
            target.display()
        )),
    }
}

/// `archi update` — the same check first, then download, unpack, verify,
/// and swap. The report names the resolved target the new binary landed
/// on; a `target` path component means a cargo build artifact was
/// replaced, and the report says so.
pub fn apply() -> Result<String, String> {
    let plat = platform()?;
    let base = base();
    let (latest_s, latest_t) = latest(&base)?;
    let current_t = triple(CURRENT).map_err(|e| format!("this binary's own version is broken: {e}"))?;
    if latest_t == current_t {
        return Ok(format!("already up to date ({CURRENT})"));
    }

    // Download and unpack in a scratch dir — nothing standing is touched
    // until the final rename.
    let scratch = Scratch::new()?;
    let name = format!("archi-{latest_s}-{plat}");
    let tarball = scratch.0.join(format!("{name}.tar.gz"));
    let url = format!("{base}/download/{name}.tar.gz");
    curl(&url, &["-o", &tarball.to_string_lossy()])
        .map_err(|e| format!("the download failed — {e}; the standing binary is untouched ({CONTINUATION})"))?;
    let tar = Command::new("tar")
        .arg("-xzf")
        .arg(&tarball)
        .arg("-C")
        .arg(&scratch.0)
        .output()
        .map_err(|e| format!("tar: {e} — the standing binary is untouched"))?;
    if !tar.status.success() {
        return Err(format!(
            "the download would not unpack — tar: {} — the standing binary is untouched",
            String::from_utf8_lossy(&tar.stderr).trim()
        ));
    }
    let unpacked = scratch.0.join(&name).join("archi");
    let whole = fs::metadata(&unpacked).map(|m| m.is_file() && m.len() > 0).unwrap_or(false);
    if !whole {
        return Err(format!(
            "the tarball unpacked without a usable {name}/archi — the standing binary is untouched"
        ));
    }

    // Resolve the running binary; canonicalize follows symlinks to their
    // target — the link itself never becomes the file.
    let exe = std::env::current_exe()
        .and_then(fs::canonicalize)
        .map_err(|e| format!("cannot resolve the running binary: {e}"))?;
    mark_executable(&unpacked)?;
    swap(&unpacked, &exe)?;

    let mut report = format!("updated {CURRENT} → {latest_s} at {}", exe.display());
    if exe.components().any(|c| c.as_os_str() == "target") {
        report.push_str(
            "\nnote: the path sits in a cargo target dir — a build artifact was replaced; \
             the installer owns ~/.local/bin",
        );
    }
    Ok(report)
}
