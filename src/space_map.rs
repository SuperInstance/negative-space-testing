//! `SpaceMap` — maps the full output space, highlighting the negative space.
//!
//! The meteorologist's blindness: if you name every cloud too quickly, there is
//! no room left for the dragon. `SpaceMap` instead asks: what shapes must NOT
//! appear? The remaining space — the positive space — is where imagination lives.

use crate::negative_test::{NegativeTest, NegativeViolation};

type ExclusionFn<T> = Box<dyn Fn(&T) -> bool>;

/// Maps the output space of a system, partitioned into positive (allowed) and
/// negative (excluded) regions.
///
/// Add samples from your system, add exclusions describing what should never
/// appear, then call [`SpaceMap::verify`] to check that no sample violated a
/// constraint.
///
/// # Examples
///
/// ```
/// use negative_space_testing::SpaceMap;
///
/// let mut map: SpaceMap<i32> = SpaceMap::new();
/// map.add_samples(0..=10);
/// map.exclude_fn("no negatives", |&v| v < 0);
/// map.exclude_fn("no values above 100", |&v| v > 100);
///
/// let result = map.verify();
/// assert!(result.is_clean());
/// assert_eq!(result.openness(), 1.0);
/// ```
pub struct SpaceMap<T> {
    samples: Vec<T>,
    exclusions: Vec<(String, ExclusionFn<T>)>,
}

/// The result of verifying a `SpaceMap`.
#[derive(Debug)]
#[must_use]
pub struct SpaceResult {
    /// All constraint violations found.
    pub violations: Vec<NegativeViolation>,
    /// Total number of samples evaluated.
    pub total_samples: usize,
    /// Number of samples in the negative space (excluded).
    pub negative_space_size: usize,
    /// Number of samples in the positive space (allowed).
    pub positive_space_size: usize,
}

impl SpaceResult {
    /// Returns `true` if no sample violated any exclusion.
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    /// Fraction of samples that fall in the positive (allowed) space.
    ///
    /// Returns `1.0` when the sample set is empty.
    pub fn openness(&self) -> f64 {
        if self.total_samples == 0 {
            1.0
        } else {
            self.positive_space_size as f64 / self.total_samples as f64
        }
    }
}

impl<T> SpaceMap<T> {
    /// Create an empty `SpaceMap`.
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            exclusions: Vec::new(),
        }
    }

    /// Add a single sample to the map.
    pub fn add_sample(&mut self, sample: T) -> &mut Self {
        self.samples.push(sample);
        self
    }

    /// Add multiple samples to the map.
    pub fn add_samples(&mut self, samples: impl IntoIterator<Item = T>) -> &mut Self {
        self.samples.extend(samples);
        self
    }

    /// Add an exclusion from a type implementing [`NegativeTest`].
    pub fn exclude<N>(&mut self, test: N) -> &mut Self
    where
        N: NegativeTest<Output = T> + 'static,
    {
        let description = test.description().to_owned();
        self.exclusions
            .push((description, Box::new(move |v| test.excludes(v))));
        self
    }

    /// Add an exclusion from a closure.
    ///
    /// The closure returns `true` for values in the negative space.
    pub fn exclude_fn(
        &mut self,
        description: impl Into<String>,
        f: impl Fn(&T) -> bool + 'static,
    ) -> &mut Self {
        self.exclusions.push((description.into(), Box::new(f)));
        self
    }

    /// Returns indices of samples that fall in the negative space.
    pub fn negative_space_indices(&self) -> Vec<usize> {
        self.samples
            .iter()
            .enumerate()
            .filter(|(_, s)| self.exclusions.iter().any(|(_, f)| f(s)))
            .map(|(i, _)| i)
            .collect()
    }

    /// Returns references to samples in the positive (allowed) space.
    pub fn positive_space(&self) -> Vec<&T> {
        self.samples
            .iter()
            .filter(|s| self.exclusions.iter().all(|(_, f)| !f(s)))
            .collect()
    }

    /// Returns references to samples in the negative (excluded) space.
    pub fn negative_space(&self) -> Vec<&T> {
        self.negative_space_indices()
            .iter()
            .map(|&i| &self.samples[i])
            .collect()
    }

    /// Verify that no sample falls in the negative space.
    ///
    /// Returns a [`SpaceResult`] describing any violations.
    pub fn verify(&self) -> SpaceResult {
        let mut violations = Vec::new();

        for (i, sample) in self.samples.iter().enumerate() {
            for (desc, f) in &self.exclusions {
                if f(sample) {
                    violations.push(NegativeViolation::new(desc.clone(), i));
                }
            }
        }

        let neg_count = self.negative_space_indices().len();
        let pos_count = self.samples.len().saturating_sub(neg_count);

        SpaceResult {
            violations,
            total_samples: self.samples.len(),
            negative_space_size: neg_count,
            positive_space_size: pos_count,
        }
    }

    /// Number of samples currently in the map.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Number of exclusions registered.
    pub fn exclusion_count(&self) -> usize {
        self.exclusions.len()
    }
}

impl<T> Default for SpaceMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::negative_test::NegativeTest;

    struct NoNegatives;
    impl NegativeTest for NoNegatives {
        type Output = i32;
        fn excludes(&self, v: &i32) -> bool { *v < 0 }
        fn description(&self) -> &str { "no negatives" }
    }

    #[test]
    fn test_new_space_map_is_empty() {
        let map: SpaceMap<i32> = SpaceMap::new();
        assert_eq!(map.sample_count(), 0);
        assert_eq!(map.exclusion_count(), 0);
    }

    #[test]
    fn test_default_is_empty() {
        let map: SpaceMap<i32> = SpaceMap::default();
        assert_eq!(map.sample_count(), 0);
    }

    #[test]
    fn test_add_sample_increments_count() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_sample(1);
        map.add_sample(2);
        assert_eq!(map.sample_count(), 2);
    }

    #[test]
    fn test_add_samples_bulk() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_samples(vec![1, 2, 3]);
        assert_eq!(map.sample_count(), 3);
    }

    #[test]
    fn test_no_exclusions_means_clean() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_samples(vec![-1, 0, 1]);
        let result = map.verify();
        assert!(result.is_clean());
    }

    #[test]
    fn test_exclusion_catches_violation() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_sample(-5);
        map.exclude_fn("no negatives", |&v| v < 0);
        let result = map.verify();
        assert!(!result.is_clean());
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].index, 0);
    }

    #[test]
    fn test_exclusion_passes_compliant_sample() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_sample(5);
        map.exclude_fn("no negatives", |&v| v < 0);
        assert!(map.verify().is_clean());
    }

    #[test]
    fn test_multiple_samples_mixed_space() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_samples(vec![-1, 2, -3, 4]);
        map.exclude_fn("no negatives", |&v| v < 0);
        let result = map.verify();
        assert!(!result.is_clean());
        assert_eq!(result.negative_space_size, 2);
        assert_eq!(result.positive_space_size, 2);
    }

    #[test]
    fn test_openness_full_clean() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_samples(vec![1, 2, 3]);
        map.exclude_fn("no negatives", |&v| v < 0);
        assert!((map.verify().openness() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_openness_with_violations() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_samples(vec![-1, 1]);
        map.exclude_fn("no negatives", |&v| v < 0);
        let result = map.verify();
        assert!((result.openness() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_openness_empty_map_is_one() {
        let map: SpaceMap<i32> = SpaceMap::new();
        assert!((map.verify().openness() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_negative_space_returns_correct_refs() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_samples(vec![-1, 2, -3]);
        map.exclude_fn("no negatives", |&v| v < 0);
        let neg = map.negative_space();
        assert_eq!(neg.len(), 2);
        assert!(neg.contains(&&-1));
        assert!(neg.contains(&&-3));
    }

    #[test]
    fn test_positive_space_returns_correct_refs() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_samples(vec![-1, 2, -3, 4]);
        map.exclude_fn("no negatives", |&v| v < 0);
        let pos = map.positive_space();
        assert_eq!(pos.len(), 2);
        assert!(pos.contains(&&2));
        assert!(pos.contains(&&4));
    }

    #[test]
    fn test_multiple_exclusions() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_samples(vec![0, 50, 200]);
        map.exclude_fn("no zeros", |&v| v == 0);
        map.exclude_fn("no values over 100", |&v| v > 100);
        let result = map.verify();
        assert_eq!(result.negative_space_size, 2);
        assert_eq!(result.positive_space_size, 1);
    }

    #[test]
    fn test_exclude_with_trait_impl() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_samples(vec![-5, 3, -1]);
        map.exclude(NoNegatives);
        let result = map.verify();
        assert_eq!(result.negative_space_size, 2);
    }

    #[test]
    fn test_violation_captures_correct_index() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_samples(vec![1, -1, 2]);
        map.exclude_fn("no negatives", |&v| v < 0);
        let result = map.verify();
        assert_eq!(result.violations[0].index, 1);
    }

    #[test]
    fn test_violation_captures_exclusion_description() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_sample(-1);
        map.exclude_fn("values must be non-negative", |&v| v < 0);
        let result = map.verify();
        assert_eq!(result.violations[0].description, "values must be non-negative");
    }

    #[test]
    fn test_empty_samples_with_exclusions_is_clean() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.exclude_fn("no negatives", |&v| v < 0);
        assert!(map.verify().is_clean());
    }

    #[test]
    fn test_chained_add_sample() {
        let mut map: SpaceMap<i32> = SpaceMap::new();
        map.add_sample(1).add_sample(2).add_sample(3);
        assert_eq!(map.sample_count(), 3);
    }

    #[test]
    fn test_string_space_map() {
        let mut map: SpaceMap<String> = SpaceMap::new();
        map.add_samples(vec!["hello".to_string(), "".to_string(), "world".to_string()]);
        map.exclude_fn("no empty strings", |s| s.is_empty());
        let result = map.verify();
        assert_eq!(result.negative_space_size, 1);
        assert_eq!(result.positive_space_size, 2);
    }
}
