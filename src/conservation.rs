//! `ConservationChecker` — verifies that conserved quantities don't decrease.
//!
//! Five hundred glazes revealed they were, at base, one mineral: iron-silicate
//! glass. The Grand Unification. Beneath variation, something is always preserved.
//! `ConservationChecker` watches for the invariants that must not be lost —
//! energy, entropy, attention, information — and reports when they slip.

/// How a quantity should change over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Monotonicity {
    /// The quantity must never decrease (e.g., entropy, accumulated knowledge).
    NonDecreasing,
    /// The quantity must never increase (e.g., error rate under improvement).
    NonIncreasing,
    /// The quantity must remain within tolerance of its initial value.
    Conserved,
}

/// Records a violation of a conservation law.
#[derive(Debug, Clone)]
#[must_use]
pub struct QuantityViolation {
    /// Name of the quantity that was violated.
    pub quantity_name: String,
    /// The step (index into the recorded values) where the violation occurred.
    pub step: usize,
    /// The value at the previous step.
    pub previous_value: f64,
    /// The value at the current step.
    pub current_value: f64,
    /// The conservation law that was violated.
    pub violation_type: Monotonicity,
}

/// The result of checking all conservation laws.
#[derive(Debug)]
#[must_use]
pub struct ConservationResult {
    /// All violations found.
    pub violations: Vec<QuantityViolation>,
    /// Number of distinct quantities checked.
    pub quantities_checked: usize,
    /// Maximum number of recorded steps across all quantities.
    pub steps_checked: usize,
}

impl ConservationResult {
    /// Returns `true` if no conservation law was violated.
    pub fn is_conserved(&self) -> bool {
        self.violations.is_empty()
    }

    /// Returns `true` if any quantity violated its conservation law.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }
}

struct Quantity {
    name: String,
    values: Vec<f64>,
    tolerance: f64,
    monotonicity: Monotonicity,
}

/// Verifies that conserved quantities (energy, entropy, attention) obey their
/// conservation laws across a sequence of recorded measurements.
///
/// # Examples
///
/// ```
/// use negative_space_testing::ConservationChecker;
///
/// let mut checker = ConservationChecker::new();
/// checker.track_non_decreasing("accuracy", 0.001);
///
/// for &val in &[0.5, 0.6, 0.7, 0.8] {
///     checker.record("accuracy", val);
/// }
///
/// assert!(checker.check().is_conserved());
/// ```
pub struct ConservationChecker {
    quantities: Vec<Quantity>,
}

impl ConservationChecker {
    /// Create an empty checker.
    pub fn new() -> Self {
        Self {
            quantities: Vec::new(),
        }
    }

    /// Register a quantity that must never decrease.
    pub fn track_non_decreasing(
        &mut self,
        name: impl Into<String>,
        tolerance: f64,
    ) -> &mut Self {
        self.quantities.push(Quantity {
            name: name.into(),
            values: Vec::new(),
            tolerance,
            monotonicity: Monotonicity::NonDecreasing,
        });
        self
    }

    /// Register a quantity that must never increase.
    pub fn track_non_increasing(
        &mut self,
        name: impl Into<String>,
        tolerance: f64,
    ) -> &mut Self {
        self.quantities.push(Quantity {
            name: name.into(),
            values: Vec::new(),
            tolerance,
            monotonicity: Monotonicity::NonIncreasing,
        });
        self
    }

    /// Register a conserved quantity that must stay within `tolerance` of its
    /// initial value.
    pub fn track_conserved(&mut self, name: impl Into<String>, tolerance: f64) -> &mut Self {
        self.quantities.push(Quantity {
            name: name.into(),
            values: Vec::new(),
            tolerance,
            monotonicity: Monotonicity::Conserved,
        });
        self
    }

    /// Record a measurement for a named quantity.
    ///
    /// Returns `true` if the quantity was found, `false` otherwise.
    pub fn record(&mut self, name: &str, value: f64) -> bool {
        match self.quantities.iter_mut().find(|q| q.name == name) {
            Some(q) => {
                q.values.push(value);
                true
            }
            None => false,
        }
    }

    /// Check all quantities for violations of their conservation laws.
    pub fn check(&self) -> ConservationResult {
        let mut violations = Vec::new();
        let mut max_steps = 0;

        for q in &self.quantities {
            max_steps = max_steps.max(q.values.len());

            for i in 1..q.values.len() {
                let prev = q.values[i - 1];
                let curr = q.values[i];

                let violated = match q.monotonicity {
                    Monotonicity::NonDecreasing => curr < prev - q.tolerance,
                    Monotonicity::NonIncreasing => curr > prev + q.tolerance,
                    Monotonicity::Conserved => {
                        let baseline = q.values[0];
                        (curr - baseline).abs() > q.tolerance
                    }
                };

                if violated {
                    violations.push(QuantityViolation {
                        quantity_name: q.name.clone(),
                        step: i,
                        previous_value: prev,
                        current_value: curr,
                        violation_type: q.monotonicity,
                    });
                }
            }
        }

        ConservationResult {
            violations,
            quantities_checked: self.quantities.len(),
            steps_checked: max_steps,
        }
    }

    /// Returns the most recently recorded value for a named quantity.
    pub fn latest(&self, name: &str) -> Option<f64> {
        self.quantities
            .iter()
            .find(|q| q.name == name)
            .and_then(|q| q.values.last().copied())
    }

    /// Returns the number of measurements recorded for a named quantity.
    pub fn measurement_count(&self, name: &str) -> Option<usize> {
        self.quantities
            .iter()
            .find(|q| q.name == name)
            .map(|q| q.values.len())
    }

    /// Returns the number of quantities being tracked.
    pub fn quantity_count(&self) -> usize {
        self.quantities.len()
    }
}

impl Default for ConservationChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_checker_has_no_quantities() {
        let c = ConservationChecker::new();
        assert_eq!(c.quantity_count(), 0);
    }

    #[test]
    fn test_default_checker_has_no_quantities() {
        let c = ConservationChecker::default();
        assert_eq!(c.quantity_count(), 0);
    }

    #[test]
    fn test_track_non_decreasing_registers() {
        let mut c = ConservationChecker::new();
        c.track_non_decreasing("entropy", 0.0);
        assert_eq!(c.quantity_count(), 1);
    }

    #[test]
    fn test_non_decreasing_passes_monotone_sequence() {
        let mut c = ConservationChecker::new();
        c.track_non_decreasing("entropy", 0.0);
        for &v in &[1.0, 2.0, 3.0, 4.0] {
            c.record("entropy", v);
        }
        assert!(c.check().is_conserved());
    }

    #[test]
    fn test_non_decreasing_fails_on_decrease() {
        let mut c = ConservationChecker::new();
        c.track_non_decreasing("entropy", 0.0);
        for &v in &[1.0, 2.0, 1.5] {
            c.record("entropy", v);
        }
        let result = c.check();
        assert!(!result.is_conserved());
        assert_eq!(result.violations[0].quantity_name, "entropy");
        assert_eq!(result.violations[0].step, 2);
    }

    #[test]
    fn test_track_non_increasing_registers() {
        let mut c = ConservationChecker::new();
        c.track_non_increasing("error_rate", 0.0);
        assert_eq!(c.quantity_count(), 1);
    }

    #[test]
    fn test_non_increasing_passes_monotone_sequence() {
        let mut c = ConservationChecker::new();
        c.track_non_increasing("error_rate", 0.0);
        for &v in &[10.0, 8.0, 5.0, 2.0] {
            c.record("error_rate", v);
        }
        assert!(c.check().is_conserved());
    }

    #[test]
    fn test_non_increasing_fails_on_increase() {
        let mut c = ConservationChecker::new();
        c.track_non_increasing("error_rate", 0.0);
        for &v in &[10.0, 8.0, 9.0] {
            c.record("error_rate", v);
        }
        assert!(!c.check().is_conserved());
    }

    #[test]
    fn test_conserved_within_tolerance() {
        let mut c = ConservationChecker::new();
        c.track_conserved("energy", 0.1);
        for &v in &[100.0, 100.05, 99.95, 100.08] {
            c.record("energy", v);
        }
        assert!(c.check().is_conserved());
    }

    #[test]
    fn test_conserved_violates_tolerance() {
        let mut c = ConservationChecker::new();
        c.track_conserved("energy", 0.1);
        for &v in &[100.0, 100.5] {
            c.record("energy", v);
        }
        assert!(!c.check().is_conserved());
    }

    #[test]
    fn test_latest_value_retrieval() {
        let mut c = ConservationChecker::new();
        c.track_non_decreasing("attention", 0.0);
        c.record("attention", 1.0);
        c.record("attention", 2.0);
        assert_eq!(c.latest("attention"), Some(2.0));
    }

    #[test]
    fn test_latest_returns_none_for_unknown() {
        let c = ConservationChecker::new();
        assert!(c.latest("unknown").is_none());
    }

    #[test]
    fn test_measurement_count() {
        let mut c = ConservationChecker::new();
        c.track_non_decreasing("score", 0.0);
        c.record("score", 1.0);
        c.record("score", 2.0);
        c.record("score", 3.0);
        assert_eq!(c.measurement_count("score"), Some(3));
    }

    #[test]
    fn test_measurement_count_none_for_unknown() {
        let c = ConservationChecker::new();
        assert!(c.measurement_count("ghost").is_none());
    }

    #[test]
    fn test_multiple_quantities_tracked_simultaneously() {
        let mut c = ConservationChecker::new();
        c.track_non_decreasing("entropy", 0.0);
        c.track_non_increasing("error", 0.0);
        c.record("entropy", 1.0);
        c.record("entropy", 2.0);
        c.record("error", 5.0);
        c.record("error", 3.0);
        assert_eq!(c.quantity_count(), 2);
        assert!(c.check().is_conserved());
    }

    #[test]
    fn test_tolerance_boundary_exact() {
        let mut c = ConservationChecker::new();
        c.track_non_decreasing("metric", 0.5);
        c.record("metric", 5.0);
        // drops by exactly tolerance — should not violate
        c.record("metric", 4.5);
        assert!(c.check().is_conserved());
    }

    #[test]
    fn test_tolerance_boundary_exceeded() {
        let mut c = ConservationChecker::new();
        c.track_non_decreasing("metric", 0.5);
        c.record("metric", 5.0);
        // drops by more than tolerance
        c.record("metric", 4.49);
        assert!(!c.check().is_conserved());
    }

    #[test]
    fn test_record_returns_false_for_unknown_quantity() {
        let mut c = ConservationChecker::new();
        assert!(!c.record("ghost", 1.0));
    }

    #[test]
    fn test_record_returns_true_for_known_quantity() {
        let mut c = ConservationChecker::new();
        c.track_non_decreasing("known", 0.0);
        assert!(c.record("known", 1.0));
    }

    #[test]
    fn test_conservation_result_has_violations_flag() {
        let mut c = ConservationChecker::new();
        c.track_non_decreasing("x", 0.0);
        c.record("x", 2.0);
        c.record("x", 1.0);
        let result = c.check();
        assert!(result.has_violations());
        assert!(!result.is_conserved());
    }

    #[test]
    fn test_single_measurement_always_conserved() {
        let mut c = ConservationChecker::new();
        c.track_non_decreasing("solo", 0.0);
        c.record("solo", 42.0);
        assert!(c.check().is_conserved());
    }
}
