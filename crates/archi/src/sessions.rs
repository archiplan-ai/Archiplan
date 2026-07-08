//! `archi session fold` — two concurrent stress rounds into one record,
//! deliberately (`requirements/stressing.md`, rounds-fold-deliberately).
//!
//! A merge can assemble a round nobody authored: two open folders after a
//! different-slug merge, or one file claimed by two charters after a
//! same-slug merge (the conflict markers git leaves are the only honest
//! boundary — fold-pressure falsified `merge=union` here, the fusion
//! commits itself). The fold is the verb-shaped repair: it keeps both
//! charters — the folded round's charter, pin and stamp land under a
//! `## Folded: <label>` heading the doc schema validates forever — and it
//! refuses what it cannot keep true: folds across pins (a stressor's ground
//! truth is its session's pin) and seals it would have to falsify. On a
//! fused *sealed* pair the surviving stamp stays and the folded stamp is
//! marked `pending remint`; `archi version remint --session` makes it true.

use std::fs;
use std::path::{Path, PathBuf};

use crate::docs::md;

/// The outcome of a fold, for the caller to report.
pub struct Folded {
    /// One line describing what folded into what.
    pub headline: String,
    /// Stressor files that moved (two-folder form), loser-relative names.
    pub moved: Vec<String>,
    /// Project-relative files to commit as one unit.
    pub files: Vec<String>,
    /// The folded stamp awaits `version remint --session`.
    pub pending_remint: bool,
}

/// Fold two rounds into one record. `into: None` normalizes a marker-fused
/// session file in place; `into: Some(winner)` folds folder `slug` into it.
pub fn fold(
    root: &Path,
    slug: &str,
    into: Option<&str>,
    note: &str,
    keep_theirs: bool,
) -> Result<Folded, String> {
    if note.trim().is_empty() {
        return Err("a fold records its why: -m <note>".into());
    }
    match into {
        Some(winner) => fold_into(root, slug, winner, note),
        None => fold_in_place(root, slug, note, keep_theirs),
    }
}

// ---- the in-place form: one file, two charters ------------------------------

/// One side of a conflicted file plus the merge's label for it.
struct Side {
    label: String,
    text: String,
}

/// Split a marker-fused file into its two whole sides. Handles diff3-style
/// conflicts (the `|||||||` base block is nobody's side).
fn split_sides(text: &str) -> Result<(Side, Side), String> {
    #[derive(PartialEq)]
    enum At {
        Common,
        Ours,
        Base,
        Theirs,
    }
    let mut at = At::Common;
    let (mut ours, mut theirs) = (String::new(), String::new());
    let (mut ours_label, mut theirs_label) = (String::new(), String::new());
    let mut conflicts = 0usize;
    for line in text.lines() {
        match at {
            At::Common if line.starts_with("<<<<<<< ") => {
                ours_label = line["<<<<<<< ".len()..].trim().to_string();
                at = At::Ours;
            }
            At::Common => {
                ours.push_str(line);
                ours.push('\n');
                theirs.push_str(line);
                theirs.push('\n');
            }
            At::Ours if line.starts_with("|||||||") => at = At::Base,
            At::Ours | At::Base if line.trim_end() == "=======" => at = At::Theirs,
            At::Ours => {
                ours.push_str(line);
                ours.push('\n');
            }
            At::Base => {}
            At::Theirs if line.starts_with(">>>>>>> ") => {
                theirs_label = line[">>>>>>> ".len()..].trim().to_string();
                conflicts += 1;
                at = At::Common;
            }
            At::Theirs => {
                theirs.push_str(line);
                theirs.push('\n');
            }
        }
    }
    if conflicts == 0 || at != At::Common {
        return Err("the file holds no complete conflict — nothing to fold in place".into());
    }
    Ok((
        Side {
            label: ours_label,
            text: ours,
        },
        Side {
            label: theirs_label,
            text: theirs,
        },
    ))
}

/// What a fold needs to know about one side: pin, seal, charter.
struct Round {
    version: String,
    closed: String,
    charter: String,
}

fn parse_round(text: &str, what: &str) -> Result<Round, String> {
    let doc = md::parse(text)
        .map_err(|e| format!("the {what} side does not parse (line {}: {})", e.line, e.message))?;
    let field = |key: &str| -> String {
        doc.frontmatter
            .as_deref()
            .and_then(|fm| fm.iter().find(|f| f.key == key))
            .map(|f| match &f.value {
                md::FieldValue::Scalar(s) => s.clone(),
                md::FieldValue::List(_) => String::new(),
            })
            .unwrap_or_default()
    };
    Ok(Round {
        version: field("version"),
        closed: field("closed"),
        charter: doc
            .summary
            .iter()
            .map(|(_, l)| l.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    })
}

/// The `## Folded:` trace appended to the survivor
/// (validated by `docs::schema::session`).
fn folded_section(label: &str, round: &Round, closed: &str, note: &str) -> String {
    format!(
        "\n## Folded: {label}\n\n{}\n\npin: {}\nclosed: {closed}\nnote: {note}\n",
        round.charter, round.version
    )
}

/// The folded side's `closed:` trailer, or why the fold cannot keep it true.
///
/// Open+open folds open; a fused sealed pair keeps the surviving stamp and
/// marks the folded one for the re-mint — identical stamps are the archive
/// collision's ambiguity (remint-consumes-the-fused-record), differing ones
/// are both already true. Mixed pairs have no honest fold.
fn folded_stamp(survivor: &Round, folded: &Round) -> Result<String, String> {
    match (survivor.closed.is_empty(), folded.closed.is_empty()) {
        (true, true) => Ok(String::new()),
        (false, false) if survivor.closed == folded.closed => Ok("pending remint".into()),
        (false, false) => Ok(folded.closed.clone()),
        _ => Err(
            "one round is sealed and one is open — a fold cannot keep that seal true; \
             split the sides by hand"
                .into(),
        ),
    }
}

fn pins_match(a: &Round, b: &Round) -> Result<(), String> {
    if a.version != b.version {
        return Err(format!(
            "a fold is only honest between rounds that pressed the same ground — \
             pins `{}` and `{}` differ; close or split by hand",
            a.version, b.version
        ));
    }
    Ok(())
}

fn anchor_of(root: &Path, slug: &str) -> Result<PathBuf, String> {
    let anchor = root
        .join("archi")
        .join("stress")
        .join(slug)
        .join(format!("{slug}.md"));
    if !anchor.is_file() {
        return Err(format!("no session `{slug}` under archi/stress/"));
    }
    Ok(anchor)
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn fold_in_place(root: &Path, slug: &str, note: &str, keep_theirs: bool) -> Result<Folded, String> {
    let anchor = anchor_of(root, slug)?;
    let text = fs::read_to_string(&anchor)
        .map_err(|e| format!("cannot read `{}`: {e}", anchor.display()))?;
    let (ours, theirs) = split_sides(&text)?;
    let (keep, fold) = if keep_theirs {
        (&theirs, &ours)
    } else {
        (&ours, &theirs)
    };
    let survivor = parse_round(&keep.text, "kept")?;
    let folded = parse_round(&fold.text, "folded")?;
    pins_match(&survivor, &folded)?;
    let stamp = folded_stamp(&survivor, &folded)?;
    let pending_remint = stamp == "pending remint";

    let mut out = keep.text.clone();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&folded_section(&fold.label, &folded, &stamp, note));
    fs::write(&anchor, out).map_err(|e| format!("cannot write `{}`: {e}", anchor.display()))?;
    Ok(Folded {
        headline: format!(
            "folded the `{}` side of `{slug}` under the `{}` charter — both kept",
            fold.label, keep.label
        ),
        moved: Vec::new(),
        files: vec![rel(root, &anchor)],
        pending_remint,
    })
}

// ---- the two-folder form: loser into winner ---------------------------------

fn fold_into(root: &Path, loser: &str, winner: &str, note: &str) -> Result<Folded, String> {
    if loser == winner {
        return Err("a session cannot fold into itself — name the other round".into());
    }
    let loser_anchor = anchor_of(root, loser)?;
    let winner_anchor = anchor_of(root, winner)?;
    let loser_text = fs::read_to_string(&loser_anchor)
        .map_err(|e| format!("cannot read `{}`: {e}", loser_anchor.display()))?;
    let winner_text = fs::read_to_string(&winner_anchor)
        .map_err(|e| format!("cannot read `{}`: {e}", winner_anchor.display()))?;
    for (slug, text) in [(loser, &loser_text), (winner, &winner_text)] {
        if crate::docs::conflict_marker_line(text).is_some() {
            return Err(format!(
                "session `{slug}` is itself marker-fused — fold it in place first: \
                 `archi session fold {slug} -m <note>`"
            ));
        }
    }
    let loser_round = parse_round(&loser_text, "loser")?;
    let winner_round = parse_round(&winner_text, "winner")?;
    if !loser_round.closed.is_empty() || !winner_round.closed.is_empty() {
        return Err(
            "two sealed rounds are two complete records and fold only when a merge already \
             fused their file; this fold is for rounds still in flight — both must be open"
                .into(),
        );
    }
    pins_match(&winner_round, &loser_round)?;

    // Every loser file but the anchor moves; a name collision is a human
    // decision (stressors are one writer's pressure), never an auto-rename.
    let loser_dir = loser_anchor.parent().expect("anchor has a folder");
    let winner_dir = winner_anchor.parent().expect("anchor has a folder");
    let mut moves: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut collisions = Vec::new();
    let mut entries: Vec<PathBuf> = fs::read_dir(loser_dir)
        .map_err(|e| format!("cannot read `{}`: {e}", loser_dir.display()))?
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    for from in entries {
        if from == loser_anchor {
            continue;
        }
        let name = from
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let to = winner_dir.join(&name);
        if to.exists() {
            collisions.push(name);
        } else {
            moves.push((from, to));
        }
    }
    if !collisions.is_empty() {
        return Err(format!(
            "both rounds hold {} — rename one side's file, then fold",
            collisions
                .iter()
                .map(|n| format!("`{n}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let mut out = winner_text.clone();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&folded_section(loser, &loser_round, "", note));
    fs::write(&winner_anchor, out)
        .map_err(|e| format!("cannot write `{}`: {e}", winner_anchor.display()))?;

    let mut moved = Vec::new();
    let mut files = vec![rel(root, &winner_anchor)];
    for (from, to) in &moves {
        fs::rename(from, to).map_err(|e| format!("cannot move `{}`: {e}", from.display()))?;
        moved.push(
            to.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
        );
        files.push(rel(root, to));
    }
    fs::remove_file(&loser_anchor)
        .map_err(|e| format!("cannot remove `{}`: {e}", loser_anchor.display()))?;
    fs::remove_dir(loser_dir)
        .map_err(|e| format!("cannot remove `{}`: {e}", loser_dir.display()))?;
    files.push(format!("archi/stress/{loser}/ (deleted)"));

    Ok(Folded {
        headline: format!(
            "folded `{loser}` into `{winner}` — {} stressor{} moved, both charters kept",
            moved.len(),
            if moved.len() == 1 { "" } else { "s" }
        ),
        moved,
        files,
        pending_remint: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FUSED: &str = "---\nversion: v0001\nclosed:\n---\n\n# Hardening\n\n\
<<<<<<< HEAD\nBob presses the storage floor.\n=======\nAlice presses the auth boundary.\n\
>>>>>>> abc1234\n";

    #[test]
    fn split_recovers_both_whole_sides() {
        let (ours, theirs) = split_sides(FUSED).unwrap();
        assert_eq!(ours.label, "HEAD");
        assert_eq!(theirs.label, "abc1234");
        assert!(ours.text.contains("Bob presses"));
        assert!(!ours.text.contains("Alice"));
        assert!(theirs.text.contains("Alice presses"));
        assert!(!theirs.text.contains("Bob"));
        // Common prefix lands on both sides.
        assert!(ours.text.starts_with("---\nversion: v0001"));
        assert!(theirs.text.starts_with("---\nversion: v0001"));
    }

    #[test]
    fn a_clean_file_has_nothing_to_fold_in_place() {
        assert!(split_sides("---\nversion: v0001\nclosed:\n---\n\n# S\n\nCharter.\n").is_err());
    }

    #[test]
    fn diff3_base_blocks_belong_to_nobody() {
        let text = "a\n<<<<<<< HEAD\nours\n||||||| merged common ancestors\nbase\n=======\n\
theirs\n>>>>>>> other\nz\n";
        let (ours, theirs) = split_sides(text).unwrap();
        assert_eq!(ours.text, "a\nours\nz\n");
        assert_eq!(theirs.text, "a\ntheirs\nz\n");
    }

    #[test]
    fn stamps_fold_by_the_seal_rules() {
        let open = |charter: &str| Round {
            version: "v0001".into(),
            closed: String::new(),
            charter: charter.into(),
        };
        let sealed = |id: &str| Round {
            version: "v0001".into(),
            closed: id.into(),
            charter: "c".into(),
        };
        assert_eq!(folded_stamp(&open("a"), &open("b")).unwrap(), "");
        assert_eq!(
            folded_stamp(&sealed("v0002"), &sealed("v0002")).unwrap(),
            "pending remint"
        );
        assert_eq!(
            folded_stamp(&sealed("v0003"), &sealed("v0002")).unwrap(),
            "v0002"
        );
        assert!(folded_stamp(&sealed("v0002"), &open("b")).is_err());
    }

    #[test]
    fn folds_across_pins_refuse() {
        let a = Round {
            version: "v0001".into(),
            closed: String::new(),
            charter: "a".into(),
        };
        let b = Round {
            version: "v0002".into(),
            closed: String::new(),
            charter: "b".into(),
        };
        let e = pins_match(&a, &b).unwrap_err();
        assert!(e.contains("same ground"), "{e}");
    }
}
