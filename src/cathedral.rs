//! `CathedralProbe` — tests the RELATIONSHIPS between components, not the components.
//!
//! From the architectural meditation: "The cathedral is not the stone. It never was.
//! It is the space the stone makes room for." A flying buttress is beautiful because
//! it is the shape that remains when you remove every stone that isn't working.
//!
//! # What This Actually Does
//!
//! Unlike simple boolean assertions, `CathedralProbe` provides **structural verification**
//! of inter-component contracts. Each probe is a named, attributed relationship test:
//!
//! - **Named** — every probe has a `relationship_name` so failures are immediately diagnostic
//! - **Attributed** — every probe records its `participants` (which components are involved)
//! - **Described** — every probe has a human-readable `description` of the expected invariant
//! - **Aggregated** — `ProbeResult` gives you `soundness_ratio()`, `violation_count()`, and
//!   a structured list of violations, not just pass/fail
//!
//! This is more than `assert!(condition)` — it's an **auditable contract system** that
//! answers: "which relationships broke, between which components, and why?"
//!
//! # Use Cases
//!
//! - **API contract testing**: verify cache/database coherence, serializer round-trips, etc.
//! - **Microservice boundary verification**: check that inter-service invariants hold
//! - **Integration testing**: verify that wiring between modules produces correct relationships
//! - **Architecture fitness**: continuously verify that structural invariants hold as code evolves
//!
//! Test the space between components. Verify that the relationships are sound.

/// A violation of a structural relationship.
#[derive(Debug, Clone)]
#[must_use]
pub struct RelationshipViolation {
    /// Name of the relationship that was violated.
    pub relationship_name: String,
    /// Names of the components participating in this relationship.
    pub participants: Vec<String>,
    /// Human-readable description of what the probe was checking.
    pub description: String,
}

/// The result of running all probes.
#[derive(Debug)]
#[must_use]
pub struct ProbeResult {
    /// All relationship violations found.
    pub violations: Vec<RelationshipViolation>,
    /// Total number of relationships verified.
    pub relationships_checked: usize,
}

impl ProbeResult {
    /// Returns `true` if all relationships are structurally sound.
    pub fn all_sound(&self) -> bool {
        self.violations.is_empty()
    }

    /// Number of violated relationships.
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }

    /// Fraction of relationships that are structurally sound.
    ///
    /// Returns `1.0` when no relationships were checked.
    pub fn soundness_ratio(&self) -> f64 {
        if self.relationships_checked == 0 {
            1.0
        } else {
            let sound = self.relationships_checked - self.violations.len();
            sound as f64 / self.relationships_checked as f64
        }
    }
}

struct Probe {
    name: String,
    participants: Vec<String>,
    description: String,
    test: Box<dyn Fn() -> bool>,
}

/// Verifies structural relationships between components — the space the stone
/// makes room for.
///
/// A `CathedralProbe` does not test whether component A works or component B
/// works. It tests whether the *relationship* between A and B has the right
/// shape — whether the flying buttress resolves the lateral thrust, whether
/// the interstice between warp threads is under the right tension.
///
/// # Examples
///
/// ```
/// use negative_space_testing::CathedralProbe;
///
/// let cache: std::collections::HashMap<&str, i32> = [("key", 42)].into();
/// let db_value = 42_i32;
///
/// let mut probe = CathedralProbe::new();
/// probe.probe(
///     "cache coherence",
///     vec!["cache", "database"],
///     "cache value must match database value",
///     move || cache.get("key").copied() == Some(db_value),
/// );
///
/// assert!(probe.verify().all_sound());
/// ```
pub struct CathedralProbe {
    probes: Vec<Probe>,
}

impl CathedralProbe {
    /// Create an empty probe set.
    pub fn new() -> Self {
        Self { probes: Vec::new() }
    }

    /// Add a probe that tests a named relationship between named participants.
    ///
    /// The `test` closure returns `true` if the relationship is sound.
    pub fn probe(
        &mut self,
        relationship_name: impl Into<String>,
        participants: Vec<impl Into<String>>,
        description: impl Into<String>,
        test: impl Fn() -> bool + 'static,
    ) -> &mut Self {
        self.probes.push(Probe {
            name: relationship_name.into(),
            participants: participants.into_iter().map(Into::into).collect(),
            description: description.into(),
            test: Box::new(test),
        });
        self
    }

    /// Add a probe with no explicit participants.
    pub fn probe_fn(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        test: impl Fn() -> bool + 'static,
    ) -> &mut Self {
        self.probe(name, Vec::<String>::new(), description, test)
    }

    /// Run all probes and return the results.
    pub fn verify(&self) -> ProbeResult {
        let mut violations = Vec::new();

        for probe in &self.probes {
            if !(probe.test)() {
                violations.push(RelationshipViolation {
                    relationship_name: probe.name.clone(),
                    participants: probe.participants.clone(),
                    description: probe.description.clone(),
                });
            }
        }

        ProbeResult {
            violations,
            relationships_checked: self.probes.len(),
        }
    }

    /// Number of probes registered.
    pub fn probe_count(&self) -> usize {
        self.probes.len()
    }
}

impl Default for CathedralProbe {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cathedral_probe_is_empty() {
        let cp = CathedralProbe::new();
        assert_eq!(cp.probe_count(), 0);
    }

    #[test]
    fn test_default_is_empty() {
        let cp = CathedralProbe::default();
        assert_eq!(cp.probe_count(), 0);
    }

    #[test]
    fn test_probe_count_increments() {
        let mut cp = CathedralProbe::new();
        cp.probe("rel1", vec!["a", "b"], "desc", || true);
        cp.probe("rel2", vec!["c", "d"], "desc", || true);
        assert_eq!(cp.probe_count(), 2);
    }

    #[test]
    fn test_passing_probe_is_all_sound() {
        let mut cp = CathedralProbe::new();
        cp.probe("healthy", vec!["x", "y"], "x < y always", || 1 < 2);
        assert!(cp.verify().all_sound());
    }

    #[test]
    fn test_failing_probe_has_violation() {
        let mut cp = CathedralProbe::new();
        cp.probe("broken", vec!["a", "b"], "impossible", || false);
        let result = cp.verify();
        assert!(!result.all_sound());
        assert_eq!(result.violation_count(), 1);
    }

    #[test]
    fn test_violation_name_is_preserved() {
        let mut cp = CathedralProbe::new();
        cp.probe("my_relationship", vec!["a"], "desc", || false);
        let result = cp.verify();
        assert_eq!(result.violations[0].relationship_name, "my_relationship");
    }

    #[test]
    fn test_violation_participants_are_preserved() {
        let mut cp = CathedralProbe::new();
        cp.probe("rel", vec!["cache", "database"], "desc", || false);
        let result = cp.verify();
        assert_eq!(
            result.violations[0].participants,
            vec!["cache", "database"]
        );
    }

    #[test]
    fn test_violation_description_is_preserved() {
        let mut cp = CathedralProbe::new();
        cp.probe("rel", vec!["a"], "must maintain coherence", || false);
        let result = cp.verify();
        assert_eq!(result.violations[0].description, "must maintain coherence");
    }

    #[test]
    fn test_multiple_probes_all_pass() {
        let mut cp = CathedralProbe::new();
        for i in 0..5 {
            cp.probe(format!("rel{i}"), vec!["a", "b"], "desc", || true);
        }
        assert!(cp.verify().all_sound());
    }

    #[test]
    fn test_multiple_probes_one_fails() {
        let mut cp = CathedralProbe::new();
        cp.probe("ok1", vec!["a"], "desc", || true);
        cp.probe("broken", vec!["b"], "desc", || false);
        cp.probe("ok2", vec!["c"], "desc", || true);
        let result = cp.verify();
        assert!(!result.all_sound());
        assert_eq!(result.violation_count(), 1);
        assert_eq!(result.relationships_checked, 3);
    }

    #[test]
    fn test_probe_fn_works() {
        let mut cp = CathedralProbe::new();
        cp.probe_fn("invariant", "sum is always positive", || 1 + 1 > 0);
        assert!(cp.verify().all_sound());
    }

    #[test]
    fn test_probe_fn_with_no_participants() {
        let mut cp = CathedralProbe::new();
        cp.probe_fn("no_parts", "desc", || false);
        let result = cp.verify();
        assert!(result.violations[0].participants.is_empty());
    }

    #[test]
    fn test_empty_probe_is_all_sound() {
        let cp = CathedralProbe::new();
        assert!(cp.verify().all_sound());
    }

    #[test]
    fn test_soundness_ratio_all_sound() {
        let mut cp = CathedralProbe::new();
        cp.probe("a", vec!["x"], "desc", || true);
        cp.probe("b", vec!["y"], "desc", || true);
        let result = cp.verify();
        assert!((result.soundness_ratio() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_soundness_ratio_half() {
        let mut cp = CathedralProbe::new();
        cp.probe("pass", vec!["x"], "desc", || true);
        cp.probe("fail", vec!["y"], "desc", || false);
        let result = cp.verify();
        assert!((result.soundness_ratio() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_soundness_ratio_empty_is_one() {
        let result = CathedralProbe::new().verify();
        assert!((result.soundness_ratio() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_probe_captures_external_values() {
        let a = 10_i32;
        let b = 20_i32;
        let mut cp = CathedralProbe::new();
        cp.probe("ordering", vec!["a", "b"], "a < b", move || a < b);
        assert!(cp.verify().all_sound());
    }

    #[test]
    fn test_chained_probe_calls() {
        let mut cp = CathedralProbe::new();
        cp.probe("r1", vec!["a"], "d1", || true)
            .probe("r2", vec!["b"], "d2", || true)
            .probe("r3", vec!["c"], "d3", || true);
        assert_eq!(cp.probe_count(), 3);
        assert!(cp.verify().all_sound());
    }
}
