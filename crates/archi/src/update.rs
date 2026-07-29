//! Self-update: `archi check-update` asks the release feed where this
//! binary stands; `archi update` swaps the binary for the feed's latest.
//!
//! The feed is GitHub releases of `archiplan-ai/Archiplan` (`ARCHI_REPO`
//! names a fork or mirror), resolved the installer's way
//! (`release/install.sh`): the `/releases/latest` redirect names the
//! newest tag and is not rate-limited; the API's `tag_name` is the
//! fallback for networks that swallow the redirect. Assets live at
//! `/releases/download/v<V>/archi-<V>-<platform>.tar.gz`, each beside its
//! `.sha256`, and the checksum must verify before anything unpacks —
//! unverifiable is not installable. `ARCHI_BASE_URL` replaces the whole
//! feed for tests and mirrors: `GET <base>/latest` is a plain text file
//! carrying `v<V>` or `<V>`, `<base>/download/v<V>/` the assets — the
//! e2e suite rides `file://` fixtures, which curl serves like any other
//! URL.
//!
//! Transport is system plumbing, the same doctrine as git in
//! `worktrees.rs`: `curl -fsSL` fetches, `shasum`/`sha256sum` hashes,
//! `tar -xzf` unpacks, and refusals carry the tool's own stderr verbatim
//! plus the continuation. The swap itself is one `rename` onto the
//! resolved running binary — atomic, so the new binary lands whole or not
//! at all: every earlier failure (torn download, bad checksum, bad
//! unpack, refused rename) leaves the standing binary untouched. Symlinks
//! resolve to their target first — the link stays a link; the file behind
//! it is what changes. The feed is the truth of latest in either
//! direction: a lower number is a rollback, and `update` follows it.

use std::fs;
use std::path::Path;
use std::process::Command;

/// The GitHub repo whose releases are the feed unless `ARCHI_REPO` says
/// otherwise.
const DEFAULT_REPO: &str = "archiplan-ai/Archiplan";

/// This binary's own version — the baseline every comparison starts from.
const CURRENT: &str = env!("CARGO_PKG_VERSION");

/// The continuation every feed-side refusal names.
const CONTINUATION: &str = "check the network, or point ARCHI_BASE_URL at a release feed";

/// Where releases come from: GitHub unless `ARCHI_BASE_URL` replaces the
/// whole feed with a plain-file mirror.
enum Feed {
    /// GitHub releases of `owner/repo` — the default lane, `ARCHI_REPO`
    /// naming a fork or mirror.
    GitHub(String),
    /// An `ARCHI_BASE_URL` mirror, trailing slash trimmed: `GET
    /// <base>/latest` is a plain text file carrying `v<V>` or `<V>`,
    /// `<base>/download/v<V>/<file>` the assets.
    Base(String),
}

impl Feed {
    fn resolve() -> Feed {
        if let Ok(v) = std::env::var("ARCHI_BASE_URL") {
            let v = v.trim();
            if !v.is_empty() {
                return Feed::Base(v.trim_end_matches('/').to_string());
            }
        }
        match std::env::var("ARCHI_REPO") {
            Ok(r) if !r.trim().is_empty() => Feed::GitHub(r.trim().to_string()),
            _ => Feed::GitHub(DEFAULT_REPO.to_string()),
        }
    }

    /// The feed's word on latest, leading `v` stripped and validated as a
    /// triple. Every refusal names the cause and the continuation.
    fn latest(&self) -> Result<(String, (u64, u64, u64)), String> {
        let v = match self {
            Feed::GitHub(repo) => github_latest(repo)?,
            Feed::Base(base) => {
                let url = format!("{base}/latest");
                let body = curl(&url, &[])
                    .map_err(|e| format!("the release feed gave no latest — {e}; {CONTINUATION}"))?;
                let raw = String::from_utf8_lossy(&body);
                let raw = raw.trim();
                raw.strip_prefix('v').unwrap_or(raw).to_string()
            }
        };
        let t =
            triple(&v).map_err(|e| format!("the feed's latest is malformed: {e}; {CONTINUATION}"))?;
        Ok((v, t))
    }

    /// The URL an asset of release `v<version>` downloads from.
    fn download_url(&self, version: &str, file: &str) -> String {
        match self {
            Feed::GitHub(repo) => {
                format!("https://github.com/{repo}/releases/download/v{version}/{file}")
            }
            Feed::Base(base) => format!("{base}/download/v{version}/{file}"),
        }
    }
}

/// The newest release of `repo` on GitHub, leading `v` stripped — the
/// installer's own two-step: the `/releases/latest` redirect names the
/// tag without rate limits; the API is the fallback for networks that
/// swallow the redirect.
fn github_latest(repo: &str) -> Result<String, String> {
    if let Ok(out) = curl(
        &format!("https://github.com/{repo}/releases/latest"),
        &["-I", "-o", "/dev/null", "-w", "%{url_effective}"],
    ) {
        let effective = String::from_utf8_lossy(&out);
        if let Some((_, tag)) = effective.trim().rsplit_once("/releases/tag/v")
            && !tag.is_empty()
        {
            return Ok(tag.to_string());
        }
    }
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let body = curl(&url, &[]).map_err(|e| {
        format!("the release feed at github.com/{repo} named no latest — {e}; {CONTINUATION}")
    })?;
    let v: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| format!("the answer at {url} is not the release JSON ({e}); {CONTINUATION}"))?;
    let Some(tag) = v.get("tag_name").and_then(|t| t.as_str()) else {
        return Err(format!("the answer at {url} carries no `tag_name`; {CONTINUATION}"));
    };
    let tag = tag.trim();
    Ok(tag.strip_prefix('v').unwrap_or(tag).to_string())
}

/// The release platform this binary was built for — the compiled-in mirror
/// of `release/install.sh`'s uname map. `Err` is the refusal for platforms
/// the release feed does not carry: Windows rides its own installer,
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
            "self-update on Windows rides the installer: `irm https://archiplan.ai/install.ps1 | iex`"
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

/// `archi check-update` — one line naming where this binary stands against
/// the feed's latest. Never touches the binary.
pub fn check() -> Result<String, String> {
    let (latest_s, latest_t) = Feed::resolve().latest()?;
    let current_t = triple(CURRENT).map_err(|e| format!("this binary's own version is broken: {e}"))?;
    Ok(match latest_t.cmp(&current_t) {
        std::cmp::Ordering::Equal => format!("archi {CURRENT} — up to date"),
        std::cmp::Ordering::Greater => {
            format!("archi {CURRENT} — newer {latest_s} available: `archi update`")
        }
        std::cmp::Ordering::Less => format!(
            "archi {CURRENT} — the feed offers {latest_s} (a rollback): `archi update` follows it"
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

/// The SHA-256 of a file through system plumbing, first-found-wins like
/// the installer: `shasum -a 256` (macOS ships it), then `sha256sum`
/// (coreutils) — the first hex field of whichever answers.
fn sha256(path: &Path) -> Result<String, String> {
    let tools: [(&str, &[&str]); 2] = [("shasum", &["-a", "256"]), ("sha256sum", &[])];
    for (tool, args) in tools {
        let out = match Command::new(tool).args(args).arg(path).output() {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("{tool}: {e} — the standing binary is untouched")),
            Ok(out) => out,
        };
        if !out.status.success() {
            return Err(format!(
                "{tool} refused the tarball: {} — the standing binary is untouched",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        let stdout = String::from_utf8_lossy(&out.stdout);
        return match stdout.split_whitespace().next() {
            Some(h) => Ok(h.to_string()),
            None => Err(format!("{tool} answered no hash — the standing binary is untouched")),
        };
    }
    Err("cannot verify the download: neither `shasum` nor `sha256sum` is on PATH — \
         install coreutils (sha256sum) or perl (shasum) — the standing binary is untouched"
        .into())
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

/// `archi update` — the same check first, then download, checksum-verify,
/// unpack, and swap. The report names the resolved target the new binary
/// landed on; a `target` path component means a cargo build artifact was
/// replaced, and the report says so.
pub fn apply() -> Result<String, String> {
    let plat = platform()?;
    let feed = Feed::resolve();
    let (latest_s, latest_t) = feed.latest()?;
    let current_t = triple(CURRENT).map_err(|e| format!("this binary's own version is broken: {e}"))?;
    if latest_t == current_t {
        return Ok(format!("already up to date ({CURRENT})"));
    }

    // Download, verify and unpack in a scratch dir — nothing standing is
    // touched until the final rename.
    let scratch = Scratch::new()?;
    let name = format!("archi-{latest_s}-{plat}");
    let file = format!("{name}.tar.gz");
    let tarball = scratch.0.join(&file);
    let url = feed.download_url(&latest_s, &file);
    curl(&url, &["-o", &tarball.to_string_lossy()])
        .map_err(|e| format!("the download failed — {e}; the standing binary is untouched ({CONTINUATION})"))?;

    // The checksum gates the unpack: beside every tarball sits its
    // `.sha256`, and unverifiable is not installable.
    let sum_url = feed.download_url(&latest_s, &format!("{file}.sha256"));
    let sum_body = curl(&sum_url, &[]).map_err(|e| {
        format!("no checksum for {file} — {e}; unverifiable is not installable — the standing binary is untouched")
    })?;
    let sum_text = String::from_utf8_lossy(&sum_body);
    let expected = sum_text
        .split_whitespace()
        .next()
        .filter(|f| f.chars().all(|c| c.is_ascii_hexdigit()))
        .ok_or_else(|| {
            format!("the checksum at {sum_url} carries no hash — unverifiable is not installable — the standing binary is untouched")
        })?;
    let actual = sha256(&tarball)?;
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(format!(
            "checksum mismatch for {file} — expected {expected}, got {actual} — the standing binary is untouched"
        ));
    }
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
