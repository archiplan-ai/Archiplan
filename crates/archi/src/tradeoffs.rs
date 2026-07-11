//! The operator's up-front priorities — `Tradeoffs`
//! (`archi/requirements/tradeoff-configuration/`). A trade-off configuration,
//! set explicitly or derived by an auto mode that polls the operator, declares
//! what the design should favour and what it may spend; the scoring read
//! consults its weighting so the landscape verdict is situated in the
//! project's own priorities rather than uniform. Absent — no
//! `archi/tradeoffs.toml`, or an empty one — the read is byte-identical to an
//! unsituated one, and never fails for lack of a configuration
//! (`archi/requirements/tradeoff-configuration/priorities-weight-the-read.md`).

use std::path::Path;

/// Where the configuration lives, relative to the project root. Its own file,
/// not `archi.toml`: operator priorities are mutated by their own verb and sit
/// beside the other operator state under `archi/`.
const CONFIG_PATH: &str = "archi/tradeoffs.toml";

/// The operator's favour/spend stance over named concerns. Empty is a valid
/// state — the read is then unweighted.
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TradeoffConfig {
    /// Concerns the design should optimize for.
    #[serde(default)]
    pub favor: Vec<String>,
    /// Concerns that may be sacrificed for the favoured ones.
    #[serde(default)]
    pub spend: Vec<String>,
}

impl TradeoffConfig {
    /// Whether the configuration holds no stance either way.
    pub fn is_empty(&self) -> bool {
        self.favor.is_empty() && self.spend.is_empty()
    }

    /// The weight coupling carries on the degree-derived neutrality read.
    /// `1.0` when empty, so an absent configuration leaves the read
    /// byte-identical. Favouring simplicity (or spending scalability) makes
    /// coupling weigh more — corridors shrink, the read presses for simpler
    /// interfaces; the reverse makes coupling weigh less.
    pub fn coupling_emphasis(&self) -> f64 {
        let favours_simplicity = self.favor.iter().any(|c| is_simplicity(c))
            || self.spend.iter().any(|c| is_scalability(c));
        let spends_simplicity = self.spend.iter().any(|c| is_simplicity(c))
            || self.favor.iter().any(|c| is_scalability(c));
        let mut e: f64 = 1.0;
        if favours_simplicity {
            e += 0.5;
        }
        if spends_simplicity {
            e -= 0.5;
        }
        e.clamp(0.5, 2.0)
    }

    /// Load the configuration from `archi/tradeoffs.toml`, leniently: an
    /// absent, unreadable or malformed file is the empty configuration, never
    /// an error — the read never fails for lack of one.
    pub fn load(root: &Path) -> Self {
        std::fs::read_to_string(root.join(CONFIG_PATH))
            .ok()
            .and_then(|t| toml::from_str(&t).ok())
            .unwrap_or_default()
    }

    /// Persist the configuration, or remove the file when empty so an empty
    /// stance leaves no trace.
    pub fn write(&self, root: &Path) -> Result<(), String> {
        let path = root.join(CONFIG_PATH);
        if self.is_empty() {
            return match std::fs::remove_file(&path) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(format!("cannot clear `{CONFIG_PATH}`: {e}")),
            };
        }
        let body =
            toml::to_string_pretty(self).map_err(|e| format!("cannot serialize tradeoffs: {e}"))?;
        std::fs::write(
            &path,
            format!("# Operator priorities weighting the scoring read (archi tradeoffs).\n{body}"),
        )
        .map_err(|e| format!("cannot write `{CONFIG_PATH}`: {e}"))
    }

    /// Derive a configuration from the operator's answers — the auto mode's
    /// poll. Each answer ranks a concern `high` (favour) or `low` (spend); any
    /// other level is ignored. The derived weighting is the one applied.
    pub fn derive(answers: &[(String, String)]) -> Self {
        let mut cfg = TradeoffConfig::default();
        for (concern, level) in answers {
            match level.trim().to_ascii_lowercase().as_str() {
                "high" | "favor" | "favour" => cfg.favor.push(concern.clone()),
                "low" | "spend" => cfg.spend.push(concern.clone()),
                _ => {}
            }
        }
        cfg
    }
}

fn is_simplicity(concern: &str) -> bool {
    matches!(
        concern.trim().to_ascii_lowercase().as_str(),
        "simplicity" | "simple"
    )
}

fn is_scalability(concern: &str) -> bool {
    matches!(
        concern.trim().to_ascii_lowercase().as_str(),
        "scalability" | "scale" | "scalable"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn simplicity_over_scalability() -> TradeoffConfig {
        TradeoffConfig {
            favor: vec!["simplicity".into()],
            spend: vec!["scalability".into()],
        }
    }

    #[test]
    fn an_empty_configuration_is_unweighted() {
        let cfg = TradeoffConfig::default();
        assert!(cfg.is_empty());
        assert_eq!(cfg.coupling_emphasis(), 1.0);
    }

    #[test]
    fn favouring_simplicity_weights_coupling_up() {
        let cfg = simplicity_over_scalability();
        assert!(!cfg.is_empty());
        assert!(cfg.coupling_emphasis() > 1.0, "{}", cfg.coupling_emphasis());
    }

    #[test]
    fn favouring_scalability_weights_coupling_down() {
        let cfg = TradeoffConfig {
            favor: vec!["scalability".into()],
            spend: vec!["simplicity".into()],
        };
        assert!(cfg.coupling_emphasis() < 1.0, "{}", cfg.coupling_emphasis());
    }

    #[test]
    fn the_auto_mode_derives_the_applied_weighting_from_answers() {
        let cfg = TradeoffConfig::derive(&[
            ("simplicity".into(), "high".into()),
            ("scalability".into(), "low".into()),
            ("cost".into(), "unsure".into()),
        ]);
        assert_eq!(cfg.favor, vec!["simplicity".to_string()]);
        assert_eq!(cfg.spend, vec!["scalability".to_string()]);
        // The derived configuration is the one whose weighting applies.
        assert_eq!(cfg.coupling_emphasis(), simplicity_over_scalability().coupling_emphasis());
    }

    #[test]
    fn load_defaults_to_empty_and_round_trips() {
        let dir = std::env::temp_dir().join(format!(
            "archi-tradeoffs-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(dir.join("archi")).unwrap();
        // Absent → empty, never an error.
        assert!(TradeoffConfig::load(&dir).is_empty());
        // Written → read back verbatim.
        let cfg = simplicity_over_scalability();
        cfg.write(&dir).unwrap();
        assert_eq!(TradeoffConfig::load(&dir), cfg);
        // Clearing removes the file, restoring the empty state.
        TradeoffConfig::default().write(&dir).unwrap();
        assert!(TradeoffConfig::load(&dir).is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }
}
