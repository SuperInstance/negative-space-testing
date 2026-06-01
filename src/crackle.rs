//! `CracklePhase` — deferred assertion checking that runs after the "cooling" phase.
//!
//! From the potter's notes: "The glaze does not crack in the heat. The crack comes
//! in the cooling." Assertions deferred during the firing phase are evaluated only
//! after the system has settled. Some cracks are failures. Some are beautiful —
//! kintsugi, the crack made permanent, the flaw elevated to the highest point of
//! the form.

/// The outcome of a single deferred assertion.
#[derive(Debug, Clone, PartialEq)]
pub enum CrackleOutcome {
    /// The assertion passed — smooth surface, no tension.
    Smooth,
    /// The assertion failed unexpectedly — a craze line, record of difference.
    Craze {
        /// Label of the failing assertion.
        label: String,
        /// Description of the failure.
        message: String,
    },
    /// The assertion failed as expected — beautiful, wabi-sabi.
    ///
    /// This is the *kintsugi* outcome: a crack that was anticipated and honored,
    /// not hidden. The failure is informative; it was the point.
    Kintsugi {
        /// Label of the expected-crack assertion.
        label: String,
    },
    /// An assertion expected to crack remained smooth — surprising but acceptable.
    UnexpectedSmooth {
        /// Label of the assertion that didn't crack.
        label: String,
    },
}

impl CrackleOutcome {
    /// Returns `true` if this outcome is acceptable (not an unexpected failure).
    pub fn is_acceptable(&self) -> bool {
        matches!(
            self,
            CrackleOutcome::Smooth | CrackleOutcome::Kintsugi { .. }
        )
    }

    /// Returns `true` if this outcome is the beautiful kintsugi crack.
    pub fn is_beautiful(&self) -> bool {
        matches!(self, CrackleOutcome::Kintsugi { .. })
    }

    /// Returns `true` if this outcome is an unexpected failure.
    pub fn is_craze(&self) -> bool {
        matches!(self, CrackleOutcome::Craze { .. })
    }
}

/// The result of cooling a `CracklePhase`.
#[derive(Debug)]
#[must_use]
pub struct CrackleResult {
    /// All outcomes, in the order their assertions were deferred.
    pub outcomes: Vec<CrackleOutcome>,
    /// Total number of assertions evaluated.
    pub total_assertions: usize,
    /// Number of unexpected failures (craze lines).
    pub cracks: usize,
    /// Number of expected failures (kintsugi).
    pub kintsugi: usize,
    /// Number of smooth (passing) assertions.
    pub smooth: usize,
}

impl CrackleResult {
    /// Returns `true` if all outcomes are acceptable (no unexpected cracks).
    pub fn all_acceptable(&self) -> bool {
        self.outcomes.iter().all(CrackleOutcome::is_acceptable)
    }

    /// Returns `true` if any unexpected failure occurred.
    pub fn has_cracks(&self) -> bool {
        self.cracks > 0
    }

    /// The fraction of assertions that resolved as beautiful kintsugi cracks.
    ///
    /// Returns `1.0` when there are no assertions.
    pub fn beauty_ratio(&self) -> f64 {
        if self.total_assertions == 0 {
            1.0
        } else {
            self.kintsugi as f64 / self.total_assertions as f64
        }
    }
}

struct DeferredCheck {
    label: String,
    check: Box<dyn FnOnce() -> bool>,
    expect_crack: bool,
}

/// Collects deferred assertions during the "firing" phase and evaluates them
/// all at once during the "cooling" phase.
///
/// # Examples
///
/// ```
/// use negative_space_testing::{CracklePhase, CrackleOutcome};
///
/// let mut phase = CracklePhase::new();
///
/// let value = 42_i32;
/// phase.defer("value is positive", move || value > 0);
/// phase.defer_crack("value is small", move || value > 1000);
///
/// let result = phase.cool();
/// assert!(result.all_acceptable());
/// assert_eq!(result.kintsugi, 1); // the expected crack was beautiful
/// ```
pub struct CracklePhase {
    deferred: Vec<DeferredCheck>,
}

impl CracklePhase {
    /// Create a new, empty phase ready for the firing.
    pub fn new() -> Self {
        Self {
            deferred: Vec::new(),
        }
    }

    /// Defer an assertion that is expected to PASS during cooling.
    ///
    /// If it fails, the outcome is [`CrackleOutcome::Craze`] — an unexpected crack.
    pub fn defer(
        &mut self,
        label: impl Into<String>,
        check: impl FnOnce() -> bool + 'static,
    ) -> &mut Self {
        self.deferred.push(DeferredCheck {
            label: label.into(),
            check: Box::new(check),
            expect_crack: false,
        });
        self
    }

    /// Defer an assertion expected to FAIL during cooling — a controlled crack.
    ///
    /// If it fails, the outcome is [`CrackleOutcome::Kintsugi`] — beautiful,
    /// expected, honored. The failure is the point.
    pub fn defer_crack(
        &mut self,
        label: impl Into<String>,
        check: impl FnOnce() -> bool + 'static,
    ) -> &mut Self {
        self.deferred.push(DeferredCheck {
            label: label.into(),
            check: Box::new(check),
            expect_crack: true,
        });
        self
    }

    /// Run all deferred assertions (the cooling phase).
    ///
    /// Consumes the phase, returning a [`CrackleResult`] with all outcomes.
    pub fn cool(self) -> CrackleResult {
        let mut outcomes = Vec::new();

        for check in self.deferred {
            let passed = (check.check)();
            let outcome = match (passed, check.expect_crack) {
                (true, false) => CrackleOutcome::Smooth,
                (false, false) => CrackleOutcome::Craze {
                    message: format!("assertion '{}' failed during cooling", check.label),
                    label: check.label,
                },
                (false, true) => CrackleOutcome::Kintsugi { label: check.label },
                (true, true) => CrackleOutcome::UnexpectedSmooth { label: check.label },
            };
            outcomes.push(outcome);
        }

        let cracks = outcomes.iter().filter(|o| o.is_craze()).count();
        let kintsugi = outcomes
            .iter()
            .filter(|o| matches!(o, CrackleOutcome::Kintsugi { .. }))
            .count();
        let smooth = outcomes
            .iter()
            .filter(|o| matches!(o, CrackleOutcome::Smooth))
            .count();
        let total = outcomes.len();

        CrackleResult {
            outcomes,
            total_assertions: total,
            cracks,
            kintsugi,
            smooth,
        }
    }

    /// Number of assertions currently deferred.
    pub fn deferred_count(&self) -> usize {
        self.deferred.len()
    }
}

impl Default for CracklePhase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_phase_has_no_deferred() {
        let phase = CracklePhase::new();
        assert_eq!(phase.deferred_count(), 0);
    }

    #[test]
    fn test_default_phase_is_empty() {
        let phase = CracklePhase::default();
        assert_eq!(phase.deferred_count(), 0);
    }

    #[test]
    fn test_defer_increments_count() {
        let mut phase = CracklePhase::new();
        phase.defer("check one", || true);
        phase.defer("check two", || true);
        assert_eq!(phase.deferred_count(), 2);
    }

    #[test]
    fn test_passing_assertion_yields_smooth() {
        let mut phase = CracklePhase::new();
        phase.defer("always true", || true);
        let result = phase.cool();
        assert_eq!(result.outcomes[0], CrackleOutcome::Smooth);
    }

    #[test]
    fn test_failing_assertion_yields_craze() {
        let mut phase = CracklePhase::new();
        phase.defer("always false", || false);
        let result = phase.cool();
        assert!(result.outcomes[0].is_craze());
    }

    #[test]
    fn test_craze_carries_label() {
        let mut phase = CracklePhase::new();
        phase.defer("my labeled check", || false);
        let result = phase.cool();
        match &result.outcomes[0] {
            CrackleOutcome::Craze { label, .. } => assert_eq!(label, "my labeled check"),
            other => panic!("expected Craze, got {other:?}"),
        }
    }

    #[test]
    fn test_expected_failure_yields_kintsugi() {
        let mut phase = CracklePhase::new();
        phase.defer_crack("beautiful crack", || false);
        let result = phase.cool();
        assert_eq!(
            result.outcomes[0],
            CrackleOutcome::Kintsugi {
                label: "beautiful crack".to_string()
            }
        );
    }

    #[test]
    fn test_expected_failure_but_passes_yields_unexpected_smooth() {
        let mut phase = CracklePhase::new();
        phase.defer_crack("unexpectedly ok", || true);
        let result = phase.cool();
        assert_eq!(
            result.outcomes[0],
            CrackleOutcome::UnexpectedSmooth {
                label: "unexpectedly ok".to_string()
            }
        );
    }

    #[test]
    fn test_mixed_outcomes() {
        let mut phase = CracklePhase::new();
        phase.defer("passes", || true);
        phase.defer("fails", || false);
        phase.defer_crack("expected crack", || false);
        let result = phase.cool();
        assert_eq!(result.smooth, 1);
        assert_eq!(result.cracks, 1);
        assert_eq!(result.kintsugi, 1);
        assert_eq!(result.total_assertions, 3);
    }

    #[test]
    fn test_all_acceptable_with_smooth_and_kintsugi() {
        let mut phase = CracklePhase::new();
        phase.defer("smooth", || true);
        phase.defer_crack("kintsugi", || false);
        assert!(phase.cool().all_acceptable());
    }

    #[test]
    fn test_all_acceptable_fails_with_craze() {
        let mut phase = CracklePhase::new();
        phase.defer("will crack", || false);
        assert!(!phase.cool().all_acceptable());
    }

    #[test]
    fn test_beauty_ratio_all_kintsugi() {
        let mut phase = CracklePhase::new();
        phase.defer_crack("crack1", || false);
        phase.defer_crack("crack2", || false);
        let result = phase.cool();
        assert!((result.beauty_ratio() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_beauty_ratio_no_kintsugi() {
        let mut phase = CracklePhase::new();
        phase.defer("smooth", || true);
        let result = phase.cool();
        assert!((result.beauty_ratio() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_beauty_ratio_empty_is_one() {
        let phase = CracklePhase::new();
        assert!((phase.cool().beauty_ratio() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_has_cracks_true() {
        let mut phase = CracklePhase::new();
        phase.defer("fails", || false);
        assert!(phase.cool().has_cracks());
    }

    #[test]
    fn test_has_cracks_false_when_all_pass() {
        let mut phase = CracklePhase::new();
        phase.defer("passes", || true);
        assert!(!phase.cool().has_cracks());
    }

    #[test]
    fn test_empty_phase_cools_cleanly() {
        let result = CracklePhase::new().cool();
        assert_eq!(result.total_assertions, 0);
        assert!(result.all_acceptable());
    }

    #[test]
    fn test_kintsugi_is_beautiful() {
        assert!(CrackleOutcome::Kintsugi {
            label: "x".to_string()
        }
        .is_beautiful());
        assert!(!CrackleOutcome::Smooth.is_beautiful());
    }

    #[test]
    fn test_closure_captures_values() {
        let threshold = 10;
        let observed = 5;
        let mut phase = CracklePhase::new();
        phase.defer("below threshold", move || observed < threshold);
        assert!(phase.cool().all_acceptable());
    }

    #[test]
    fn test_multiple_deferred_all_pass() {
        let mut phase = CracklePhase::new();
        for i in 0..10 {
            phase.defer(format!("check {i}"), move || i < 100);
        }
        let result = phase.cool();
        assert_eq!(result.smooth, 10);
        assert!(result.all_acceptable());
    }

    #[test]
    fn test_outcome_is_acceptable_smooth() {
        assert!(CrackleOutcome::Smooth.is_acceptable());
    }

    #[test]
    fn test_outcome_is_acceptable_kintsugi() {
        assert!(CrackleOutcome::Kintsugi {
            label: "x".into()
        }
        .is_acceptable());
    }

    #[test]
    fn test_outcome_not_acceptable_craze() {
        assert!(!CrackleOutcome::Craze {
            label: "x".into(),
            message: "fail".into()
        }
        .is_acceptable());
    }
}
