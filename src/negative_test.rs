//! The `NegativeTest` trait — define constraints on what code should NOT produce.
//!
//! Like the Jacquard card: the pattern is encoded in the absences, not the presences.
//! Each hole in the card is a place where the thread must rise. Each `NegativeTest`
//! is a shape the output must not take.

/// A constraint on what a system should NOT produce.
///
/// The child sees the dragon; the meteorologist sees *cumulus mediocris*. A
/// `NegativeTest` is a meteorologist you choose NOT to listen to — it names
/// the clouds you want to exclude so the imagination has room to move in the
/// remaining space.
///
/// # Examples
///
/// ```
/// use negative_space_testing::NegativeTest;
///
/// struct NoNegatives;
///
/// impl NegativeTest for NoNegatives {
///     type Output = i32;
///     fn excludes(&self, value: &i32) -> bool { *value < 0 }
///     fn description(&self) -> &str { "output must not be negative" }
/// }
///
/// let test = NoNegatives;
/// assert!(test.excludes(&-1));
/// assert!(!test.excludes(&42));
/// ```
pub trait NegativeTest {
    /// The type of value being tested.
    type Output;

    /// Returns `true` if this value is in the negative space — i.e., it should
    /// NOT have been produced.
    fn excludes(&self, value: &Self::Output) -> bool;

    /// Human-readable description of what this exclusion forbids.
    fn description(&self) -> &str;
}

/// Records a single negative-space violation: a value that should not exist.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct NegativeViolation {
    /// The exclusion that was violated.
    pub description: String,
    /// Index of the offending sample in the collection.
    pub index: usize,
}

impl NegativeViolation {
    /// Create a new violation record.
    pub fn new(description: impl Into<String>, index: usize) -> Self {
        Self {
            description: description.into(),
            index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoNegatives;
    impl NegativeTest for NoNegatives {
        type Output = i32;
        fn excludes(&self, v: &i32) -> bool { *v < 0 }
        fn description(&self) -> &str { "must not be negative" }
    }

    struct NoEmptyStrings;
    impl NegativeTest for NoEmptyStrings {
        type Output = String;
        fn excludes(&self, v: &String) -> bool { v.is_empty() }
        fn description(&self) -> &str { "string must not be empty" }
    }

    #[derive(Debug)]
    struct Point { x: f64, y: f64 }

    struct NotAtOrigin;
    impl NegativeTest for NotAtOrigin {
        type Output = Point;
        fn excludes(&self, v: &Point) -> bool { v.x == 0.0 && v.y == 0.0 }
        fn description(&self) -> &str { "point must not be at origin" }
    }

    #[test]
    fn test_excludes_returns_true_for_excluded_value() {
        let test = NoNegatives;
        assert!(test.excludes(&-1));
        assert!(test.excludes(&-100));
    }

    #[test]
    fn test_excludes_returns_false_for_allowed_value() {
        let test = NoNegatives;
        assert!(!test.excludes(&0));
        assert!(!test.excludes(&42));
    }

    #[test]
    fn test_description_is_correct() {
        let test = NoNegatives;
        assert_eq!(test.description(), "must not be negative");
    }

    #[test]
    fn test_string_negative_test_excludes_empty() {
        let test = NoEmptyStrings;
        assert!(test.excludes(&String::new()));
    }

    #[test]
    fn test_string_negative_test_allows_non_empty() {
        let test = NoEmptyStrings;
        assert!(!test.excludes(&"hello".to_string()));
    }

    #[test]
    fn test_custom_type_negative_test() {
        let test = NotAtOrigin;
        assert!(test.excludes(&Point { x: 0.0, y: 0.0 }));
        assert!(!test.excludes(&Point { x: 1.0, y: 0.0 }));
    }

    #[test]
    fn test_negative_violation_stores_description() {
        let v = NegativeViolation::new("must not be zero", 3);
        assert_eq!(v.description, "must not be zero");
        assert_eq!(v.index, 3);
    }

    #[test]
    fn test_negative_violation_equality() {
        let a = NegativeViolation::new("same", 0);
        let b = NegativeViolation::new("same", 0);
        assert_eq!(a, b);
    }

    #[test]
    fn test_multiple_negative_tests_compose() {
        let no_neg = NoNegatives;
        let values = vec![-1, 0, 5];
        let excluded: Vec<_> = values.iter().filter(|&&v| no_neg.excludes(&v)).collect();
        assert_eq!(excluded, vec![&-1]);
    }
}
