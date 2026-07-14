//! The fixed trade-off vocabulary (`archi/requirements/spec-docs/a-decision-prices-the-fork.md`):
//! nine axes a decision can prefer or sacrifice, plus `Other` for any
//! off-list label, kept verbatim. The set is code-defined and closed —
//! editing the nine is a release-level migration, never a project edit.
//! Parsing is total: an off-list label is never a reject, it is `Other`,
//! surfaced by `check` as an `off_list_axis` finding; a large `Other` share
//! is the signal the fixed nine no longer fit the project.

/// One trade-off axis: a name for what a decision buys or pays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Axis {
    /// Fewest moving parts; easy to hold in your head and change.
    Simplicity,
    /// Latency / throughput on the happy path.
    Performance,
    /// Headroom to grow with load, data, or users.
    Scalability,
    /// Stays available and recovers under failure.
    Reliability,
    /// Protects confidentiality and integrity against adversaries.
    Security,
    /// Results are accurate / consistent in steady state.
    Correctness,
    /// Cheap to extend or rework later.
    Evolvability,
    /// Easy to run, observe, and recover; low on-call burden.
    Operability,
    /// Money and resource efficiency, build and run.
    Cost,
    /// Off-list label, preserved verbatim. Never equal to a known axis.
    Other(String),
}

impl Axis {
    /// The canonical wire names of the fixed nine, in display order.
    pub const KNOWN: [&'static str; 9] = [
        "simplicity",
        "performance",
        "scalability",
        "reliability",
        "security",
        "correctness",
        "evolvability",
        "operability",
        "cost",
    ];

    /// Total: trimmed, case-insensitive; anything off-list is `Other` with
    /// the trimmed raw label — surfaced downstream, never a parse failure.
    pub fn parse(raw: &str) -> Axis {
        let trimmed = raw.trim();
        match trimmed.to_ascii_lowercase().as_str() {
            "simplicity" => Axis::Simplicity,
            "performance" => Axis::Performance,
            "scalability" => Axis::Scalability,
            "reliability" => Axis::Reliability,
            "security" => Axis::Security,
            "correctness" => Axis::Correctness,
            "evolvability" => Axis::Evolvability,
            "operability" => Axis::Operability,
            "cost" => Axis::Cost,
            _ => Axis::Other(trimmed.to_string()),
        }
    }

    /// The wire form — a known axis's canonical lowercase name, an `Other`'s
    /// label verbatim. `parse(a.wire()) == a` for every axis.
    pub fn wire(&self) -> &str {
        match self {
            Axis::Simplicity => "simplicity",
            Axis::Performance => "performance",
            Axis::Scalability => "scalability",
            Axis::Reliability => "reliability",
            Axis::Security => "security",
            Axis::Correctness => "correctness",
            Axis::Evolvability => "evolvability",
            Axis::Operability => "operability",
            Axis::Cost => "cost",
            Axis::Other(label) => label,
        }
    }

    /// One-line definition, shipped with the set so `archi axes` teaches it.
    pub fn definition(&self) -> &'static str {
        match self {
            Axis::Simplicity => "fewest moving parts; easy to hold in your head and change",
            Axis::Performance => "latency / throughput on the happy path",
            Axis::Scalability => "headroom to grow with load, data, or users",
            Axis::Reliability => "stays available and recovers under failure",
            Axis::Security => "protects confidentiality and integrity against adversaries",
            Axis::Correctness => {
                "results are accurate / consistent in steady state (distinct from reliability)"
            }
            Axis::Evolvability => "cheap to extend or rework later",
            Axis::Operability => "easy to run, observe, and recover; low on-call burden",
            Axis::Cost => "money and resource efficiency, build and run",
            Axis::Other(_) => "off-list trade-off; preserves the raw label (set-fit diagnostic)",
        }
    }
}

/// Parse a raw frontmatter list, entry-wise.
pub fn parse_list(raw: &[String]) -> Vec<Axis> {
    raw.iter().map(|s| Axis::parse(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_is_total_and_case_forgiving() {
        assert_eq!(Axis::parse("  COST "), Axis::Cost);
        assert_eq!(Axis::parse("Simplicity"), Axis::Simplicity);
        // Off-list is Other with the trimmed raw label, never a reject.
        assert_eq!(
            Axis::parse(" audit-trail "),
            Axis::Other("audit-trail".to_string())
        );
        // A near-miss synonym is off-list too: the nine are canonical names.
        assert_eq!(Axis::parse("simple"), Axis::Other("simple".to_string()));
    }

    #[test]
    fn wire_round_trips_every_axis() {
        for name in Axis::KNOWN {
            let a = Axis::parse(name);
            assert!(!matches!(a, Axis::Other(_)), "{name} is a known axis");
            assert_eq!(a.wire(), name);
            assert_eq!(Axis::parse(a.wire()), a);
        }
        let other = Axis::Other("Weird Label".to_string());
        assert_eq!(other.wire(), "Weird Label");
        assert_eq!(Axis::parse(other.wire()), other);
    }

    #[test]
    fn the_set_ships_its_definitions() {
        for name in Axis::KNOWN {
            assert!(!Axis::parse(name).definition().is_empty());
        }
        assert!(Axis::Other(String::new()).definition().contains("off-list"));
    }
}
