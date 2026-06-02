//! # Negative Space Testing
//!
//! A testing framework where you define what code DOES NOT do.
//!
//! Inspired by the insight: "The meteorologist knows the cloud names too quickly —
//! over-specification kills imagination." Traditional testing specifies what code
//! *should* do. This crate tests the negative space — what code *must never* do.
//!
//! ## Core Concepts
//!
//! - **Forbidden**: Define behaviors that must never occur
//! - **SpaceMap**: Map the full output space, highlighting negative space
//! - **ConservationChecker**: Verify conserved quantities don't decrease
//! - **CracklePhase**: Deferred assertions that check after a "cooling" period
//! - **CathedralProbe**: Test the SPACE between components, not the components

#![deny(unsafe_code)]

pub mod topology;

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

// ─── Forbidden ───────────────────────────────────────────────────────

/// A forbidden behavior — something the code must never produce.
impl<T> std::fmt::Debug for Forbidden<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Forbidden")
            .field("description", &self.description)
            .finish_non_exhaustive()
    }
}

pub struct Forbidden<T> {
    /// Human-readable description of what's forbidden
    pub description: String,
    /// The predicate that returns true when the forbidden behavior occurs
    pub predicate: Box<dyn Fn(&T) -> bool + Send + Sync>,
}

impl<T> Forbidden<T> {
    /// Create a new forbidden behavior
    pub fn new<F>(description: &str, predicate: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Self {
            description: description.to_string(),
            predicate: Box::new(predicate),
        }
    }

    /// Check if a value triggers this forbidden behavior
    pub fn is_triggered(&self, value: &T) -> bool {
        (self.predicate)(value)
    }
}

/// Result of checking a value against a set of forbidden behaviors
#[derive(Debug, Clone)]
pub struct NegativeResult {
    /// Which forbidden behaviors were triggered
    pub violations: Vec<String>,
    /// Total values checked
    pub total_checked: usize,
    /// Values that passed all forbidden checks
    pub clean_count: usize,
}

impl NegativeResult {
    /// Returns true if no forbidden behaviors were triggered
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    /// Returns the ratio of clean values to total checked
    pub fn clean_ratio(&self) -> f64 {
        if self.total_checked == 0 {
            1.0
        } else {
            self.clean_count as f64 / self.total_checked as f64
        }
    }
}

/// A negative space test — verifies what code does NOT do
pub struct NegativeTest<T> {
    forbidden: Vec<Forbidden<T>>,
}

impl<T> NegativeTest<T> {
    /// Create a new negative test with no forbidden behaviors
    pub fn new() -> Self {
        Self {
            forbidden: Vec::new(),
        }
    }

    /// Add a forbidden behavior
    pub fn forbid<F>(mut self, description: &str, predicate: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.forbidden.push(Forbidden::new(description, predicate));
        self
    }

    /// Check a single value against all forbidden behaviors
    pub fn check(&self, value: &T) -> NegativeResult {
        let violations: Vec<String> = self
            .forbidden
            .iter()
            .filter(|f| f.is_triggered(value))
            .map(|f| f.description.clone())
            .collect();
        let clean = violations.is_empty();
        NegativeResult {
            violations,
            total_checked: 1,
            clean_count: if clean { 1 } else { 0 },
        }
    }

    /// Check multiple values against all forbidden behaviors
    pub fn check_all<'a, I>(&self, values: I) -> NegativeResult
    where
        I: IntoIterator<Item = &'a T>,
        T: 'a,
    {
        let mut violations = Vec::new();
        let mut total = 0;
        let mut clean = 0;
        for value in values {
            total += 1;
            let v: Vec<String> = self
                .forbidden
                .iter()
                .filter(|f| f.is_triggered(value))
                .map(|f| f.description.clone())
                .collect();
            if v.is_empty() {
                clean += 1;
            } else {
                violations.extend(v);
            }
        }
        NegativeResult {
            violations,
            total_checked: total,
            clean_count: clean,
        }
    }

    /// Returns the number of forbidden behaviors registered
    pub fn forbidden_count(&self) -> usize {
        self.forbidden.len()
    }
}

impl<T> Default for NegativeTest<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ─── SpaceMap ────────────────────────────────────────────────────────

/// A map of the full output space, showing what's occupied and what's negative space
#[derive(Debug, Clone)]
pub struct SpaceMap<K, V> {
    /// Occupied regions of the space
    occupied: HashMap<K, V>,
    /// Negative space — regions that must remain empty
    forbidden_keys: HashSet<K>,
}

impl<K, V> SpaceMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new empty space map
    pub fn new() -> Self {
        Self {
            occupied: HashMap::new(),
            forbidden_keys: HashSet::new(),
        }
    }

    /// Mark a region as occupied
    pub fn occupy(&mut self, key: K, value: V) {
        self.occupied.insert(key, value);
    }

    /// Mark a region as forbidden (negative space)
    pub fn forbid(&mut self, key: K) {
        self.forbidden_keys.insert(key);
    }

    /// Check if any occupied region intrudes on forbidden space
    pub fn check_intrusions(&self) -> Vec<K> {
        self.occupied
            .keys()
            .filter(|k| self.forbidden_keys.contains(k))
            .cloned()
            .collect()
    }

    /// Returns the ratio of forbidden space that remains unoccupied
    pub fn negative_space_ratio(&self) -> f64 {
        if self.forbidden_keys.is_empty() {
            return 1.0;
        }
        let intrusions = self.check_intrusions().len();
        1.0 - (intrusions as f64 / self.forbidden_keys.len() as f64)
    }

    /// Returns the number of occupied regions
    pub fn occupied_count(&self) -> usize {
        self.occupied.len()
    }

    /// Returns the number of forbidden regions
    pub fn forbidden_count(&self) -> usize {
        self.forbidden_keys.len()
    }

    /// Get a value from occupied space
    pub fn get(&self, key: &K) -> Option<&V> {
        self.occupied.get(key)
    }
}

impl<K, V> Default for SpaceMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

// ─── ConservationChecker ─────────────────────────────────────────────

/// A quantity that should be conserved across operations
#[derive(Debug, Clone)]
pub struct ConservedQuantity {
    pub name: String,
    pub value: f64,
    pub tolerance: f64,
}

impl ConservedQuantity {
    pub fn new(name: &str, value: f64, tolerance: f64) -> Self {
        Self {
            name: name.to_string(),
            value,
            tolerance,
        }
    }

    /// Check if this quantity is conserved relative to an initial value.
    /// One-sided: allows increase, only flags decrease beyond tolerance.
    pub fn is_conserved(&self, initial: f64) -> bool {
        self.value >= initial - self.tolerance
    }
}

/// Tracks conserved quantities across operations
pub struct ConservationChecker {
    initial: HashMap<String, (f64, f64)>, // (value, tolerance)
    current: HashMap<String, f64>,
    history: Vec<HashMap<String, f64>>,
}

impl ConservationChecker {
    /// Create a new conservation checker
    pub fn new() -> Self {
        Self {
            initial: HashMap::new(),
            current: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// Register a conserved quantity with its initial value and tolerance
    pub fn register(&mut self, name: &str, initial_value: f64, tolerance: f64) {
        self.initial.insert(name.to_string(), (initial_value, tolerance));
        self.current.insert(name.to_string(), initial_value);
    }

    /// Update a quantity's current value
    pub fn update(&mut self, name: &str, value: f64) {
        if let Some(current) = self.current.get_mut(name) {
            *current = value;
        }
    }

    /// Snapshot current state into history
    pub fn snapshot(&mut self) {
        self.history.push(self.current.clone());
    }

    /// Check all conserved quantities
    pub fn check(&self) -> Vec<ConservedQuantity> {
        self.current
            .iter()
            .map(|(name, value)| {
                let (_initial, tolerance) = self.initial.get(name).copied().unwrap_or((*value, 0.0));
                ConservedQuantity {
                    name: name.clone(),
                    value: *value,
                    tolerance,
                }
            })
            .collect()
    }

    /// Check if a specific quantity is conserved
    pub fn is_conserved(&self, name: &str) -> bool {
        if let (Some((initial, tolerance)), Some(current)) =
            (self.initial.get(name), self.current.get(name))
        {
            *current >= *initial - *tolerance
        } else {
            true
        }
    }

    /// Returns names of all violated conservation laws
    pub fn violations(&self) -> Vec<String> {
        self.current
            .keys()
            .filter(|name| !self.is_conserved(name))
            .cloned()
            .collect()
    }

    /// Returns the history length
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Get a quantity's value from a specific history index
    pub fn history_value(&self, index: usize, name: &str) -> Option<f64> {
        self.history.get(index).and_then(|m| m.get(name).copied())
    }
}

impl Default for ConservationChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ─── CracklePhase ────────────────────────────────────────────────────

/// A deferred assertion that is checked during a "cooling" phase after execution.
/// Inspired by crackle glaze — patterns form during cooling, not firing.
pub struct CracklePhase<T> {
    /// Assertions to check during cooling
    cooling_assertions: Vec<CoolingAssertion<T>>,
    /// Values accumulated during firing
    accumulated: Vec<T>,
    /// Whether the cooling phase has been run
    cooled: bool,
}

struct CoolingAssertion<T> {
    description: String,
    #[allow(clippy::type_complexity)]
    check: Box<dyn Fn(&[T]) -> bool + Send + Sync>,
}

impl<T> CracklePhase<T> {
    /// Create a new crackle phase
    pub fn new() -> Self {
        Self {
            cooling_assertions: Vec::new(),
            accumulated: Vec::new(),
            cooled: false,
        }
    }

    /// Add a cooling-phase assertion
    pub fn on_cool<F>(mut self, description: &str, check: F) -> Self
    where
        F: Fn(&[T]) -> bool + Send + Sync + 'static,
    {
        self.cooling_assertions.push(CoolingAssertion {
            description: description.to_string(),
            check: Box::new(check),
        });
        self
    }

    /// Accumulate a value during the firing phase
    pub fn fire(&mut self, value: T) {
        self.accumulated.push(value);
        self.cooled = false;
    }

    /// Run the cooling phase and check all deferred assertions
    pub fn cool(&mut self) -> CrackleResult {
        self.cooled = true;
        let mut violations = Vec::new();
        for assertion in &self.cooling_assertions {
            if !(assertion.check)(&self.accumulated) {
                violations.push(assertion.description.clone());
            }
        }
        CrackleResult {
            total_values: self.accumulated.len(),
            assertion_count: self.cooling_assertions.len(),
            violations,
        }
    }

    /// Returns the number of accumulated values
    pub fn len(&self) -> usize {
        self.accumulated.len()
    }

    /// Returns true if no values have been accumulated
    pub fn is_empty(&self) -> bool {
        self.accumulated.is_empty()
    }
}

impl<T> Default for CracklePhase<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a cooling phase
#[derive(Debug, Clone)]
pub struct CrackleResult {
    pub total_values: usize,
    pub assertion_count: usize,
    pub violations: Vec<String>,
}

impl CrackleResult {
    /// Returns true if all cooling assertions passed
    pub fn is_sound(&self) -> bool {
        self.violations.is_empty()
    }
}

// ─── CathedralProbe ──────────────────────────────────────────────────

/// Tests the SPACE between components, not the components themselves.
/// Inspired by "the cathedral is not the stone — it is the space the stone makes room for."
pub struct CathedralProbe {
    /// Component names
    components: Vec<String>,
    /// Edges (connections) between components with weights
    edges: HashMap<(String, String), f64>,
}

impl CathedralProbe {
    /// Create a new cathedral probe with named components
    pub fn new(components: Vec<&str>) -> Self {
        Self {
            components: components.iter().map(|s| s.to_string()).collect(),
            edges: HashMap::new(),
        }
    }

    /// Add a connection between two components
    pub fn connect(&mut self, a: &str, b: &str, weight: f64) {
        self.edges.insert((a.to_string(), b.to_string()), weight);
    }

    /// Compute the Laplacian spectrum of the component graph.
    /// The spectrum reveals the "space between components."
    pub fn spectrum(&self) -> Vec<f64> {
        let n = self.components.len();
        if n == 0 {
            return Vec::new();
        }
        // Build adjacency matrix
        let mut adj = vec![vec![0.0f64; n]; n];
        let mut degree = vec![0.0f64; n];
        for ((a, b), w) in &self.edges {
            let i = self.components.iter().position(|c| c == a).unwrap_or(0);
            let j = self.components.iter().position(|c| c == b).unwrap_or(0);
            adj[i][j] += w;
            adj[j][i] += w;
            degree[i] += w;
            degree[j] += w;
        }
        // Build Laplacian: L = D - A
        let mut lap = vec![vec![0.0f64; n]; n];
        for i in 0..n {
            for j in 0..n {
                lap[i][j] = if i == j { degree[i] } else { -adj[i][j] };
            }
        }
        // Compute eigenvalues via power iteration for each
        // For small matrices, use a simple Jacobi-like approach
        // For robustness, compute characteristic polynomial coefficients
        if n == 1 {
            return vec![lap[0][0]];
        }
        if n == 2 {
            let trace = lap[0][0] + lap[1][1];
            let det = lap[0][0] * lap[1][1] - lap[0][1] * lap[1][0];
            let disc = (trace * trace - 4.0 * det).max(0.0).sqrt();
            let mut eigs = vec![(trace + disc) / 2.0, (trace - disc) / 2.0];
            eigs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            return eigs;
        }
        // For n >= 3: QR algorithm (simplified)
        let mut mat = lap;
        for _ in 0..100 {
            // QR decomposition via Gram-Schmidt
            let (q, r) = qr_decompose(&mat, n);
            mat = mat_mul(&r, &q, n);
        }
        let mut eigs: Vec<f64> = (0..n).map(|i| mat[i][i]).collect();
        eigs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        eigs
    }

    /// Returns the Fiedler value (second-smallest eigenvalue).
    /// Higher = better connected space between components.
    pub fn fiedler_value(&self) -> f64 {
        let spec = self.spectrum();
        if spec.len() >= 2 {
            spec[1]
        } else {
            0.0
        }
    }

    /// Returns the Cheeger constant approximation.
    /// Measures the "bottleneck" in the space between components.
    pub fn cheeger_constant(&self) -> f64 {
        let fiedler = self.fiedler_value();
        let n = self.components.len() as f64;
        if n <= 1.0 {
            return 0.0;
        }
        (fiedler / 2.0).clamp(0.0, 1.0)
    }

    /// Check if the space between components is healthy
    pub fn is_healthy(&self, min_fiedler: f64) -> bool {
        self.fiedler_value() >= min_fiedler
    }

    /// Returns the number of components
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Returns the number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

/// QR decomposition for eigenvalue computation
fn qr_decompose(mat: &[Vec<f64>], n: usize) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut q = vec![vec![0.0; n]; n];
    let mut r = vec![vec![0.0; n]; n];
    let mut v = vec![vec![0.0; n]; n];
    for j in 0..n {
        for i in 0..n {
            v[i][j] = mat[i][j];
        }
        for i in 0..j {
            let dot: f64 = (0..n).map(|k| mat[k][j] * q[k][i]).sum();
            r[i][j] = dot;
            for k in 0..n {
                v[k][j] -= dot * q[k][i];
            }
        }
        let norm: f64 = (0..n).map(|k| v[k][j] * v[k][j]).sum::<f64>().sqrt();
        r[j][j] = if norm < 1e-12 { 0.0 } else { norm };
        for k in 0..n {
            q[k][j] = if norm < 1e-12 { 0.0 } else { v[k][j] / norm };
        }
    }
    (q, r)
}

fn mat_mul(a: &[Vec<f64>], b: &[Vec<f64>], n: usize) -> Vec<Vec<f64>> {
    let mut c = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    c
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // === Forbidden Tests ===

    #[test]
    fn test_forbidden_basic() {
        let f = Forbidden::new("negative value", |x: &i32| *x < 0);
        assert!(f.is_triggered(&-1));
        assert!(!f.is_triggered(&5));
    }

    #[test]
    fn test_forbidden_description() {
        let f = Forbidden::new("too large", |x: &i32| *x > 100);
        assert_eq!(f.description, "too large");
    }

    #[test]
    fn test_negative_test_single_value() {
        let nt = NegativeTest::<i32>::new()
            .forbid("negative", |x| *x < 0)
            .forbid("over 100", |x| *x > 100);
        let result = nt.check(&50);
        assert!(result.is_clean());
        assert_eq!(result.total_checked, 1);
    }

    #[test]
    fn test_negative_test_violation() {
        let nt = NegativeTest::<i32>::new().forbid("negative", |x| *x < 0);
        let result = nt.check(&-5);
        assert!(!result.is_clean());
        assert!(result.violations.contains(&"negative".to_string()));
    }

    #[test]
    fn test_negative_test_multiple_forbids() {
        let nt = NegativeTest::<i32>::new()
            .forbid("negative", |x| *x < 0)
            .forbid("over 100", |x| *x > 100)
            .forbid("zero", |x| *x == 0);
        assert_eq!(nt.forbidden_count(), 3);
        let result = nt.check(&0);
        assert!(!result.is_clean());
        assert!(result.violations.contains(&"zero".to_string()));
    }

    #[test]
    fn test_negative_test_all_values() {
        let nt = NegativeTest::<i32>::new().forbid("negative", |x| *x < 0);
        let values = vec![1, 2, 3, 4, 5];
        let result = nt.check_all(&values);
        assert!(result.is_clean());
        assert_eq!(result.total_checked, 5);
        assert_eq!(result.clean_count, 5);
    }

    #[test]
    fn test_negative_test_mixed_values() {
        let nt = NegativeTest::<i32>::new().forbid("negative", |x| *x < 0);
        let values = vec![1, -1, 3, -5, 5];
        let result = nt.check_all(&values);
        assert!(!result.is_clean());
        assert_eq!(result.total_checked, 5);
        assert_eq!(result.clean_count, 3);
    }

    #[test]
    fn test_negative_test_clean_ratio() {
        let nt = NegativeTest::<i32>::new().forbid("negative", |x| *x < 0);
        let values = vec![1, -1, 3, -5, 5];
        let result = nt.check_all(&values);
        assert!((result.clean_ratio() - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_negative_test_empty() {
        let nt = NegativeTest::<i32>::new();
        let result = nt.check(&42);
        assert!(result.is_clean());
    }

    // === SpaceMap Tests ===

    #[test]
    fn test_space_map_basic() {
        let mut sm = SpaceMap::<&str, i32>::new();
        sm.occupy("a", 1);
        sm.occupy("b", 2);
        assert_eq!(sm.occupied_count(), 2);
        assert_eq!(*sm.get(&"a").unwrap(), 1);
    }

    #[test]
    fn test_space_map_forbidden() {
        let mut sm = SpaceMap::<&str, i32>::new();
        sm.forbid("secret");
        sm.occupy("public", 1);
        assert_eq!(sm.forbidden_count(), 1);
        assert!(sm.check_intrusions().is_empty());
    }

    #[test]
    fn test_space_map_intrusion() {
        let mut sm = SpaceMap::<&str, i32>::new();
        sm.forbid("secret");
        sm.occupy("secret", 42);
        let intrusions = sm.check_intrusions();
        assert_eq!(intrusions.len(), 1);
    }

    #[test]
    fn test_space_map_negative_ratio() {
        let mut sm = SpaceMap::<&str, i32>::new();
        sm.forbid("a");
        sm.forbid("b");
        sm.forbid("c");
        sm.occupy("a", 1); // intrudes on forbidden
        let ratio = sm.negative_space_ratio();
        assert!((ratio - (2.0 / 3.0)).abs() < 0.01);
    }

    #[test]
    fn test_space_map_all_clean() {
        let mut sm = SpaceMap::<&str, i32>::new();
        sm.forbid("a");
        sm.forbid("b");
        sm.occupy("c", 1);
        sm.occupy("d", 2);
        assert!(sm.check_intrusions().is_empty());
        assert!((sm.negative_space_ratio() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_space_map_no_forbidden() {
        let mut sm = SpaceMap::<&str, i32>::new();
        sm.occupy("a", 1);
        assert!((sm.negative_space_ratio() - 1.0).abs() < 0.01);
    }

    // === ConservationChecker Tests ===

    #[test]
    fn test_conservation_register() {
        let mut cc = ConservationChecker::new();
        cc.register("energy", 100.0, 1.0);
        assert!(cc.is_conserved("energy"));
    }

    #[test]
    fn test_conservation_decrease_violation() {
        let mut cc = ConservationChecker::new();
        cc.register("energy", 100.0, 1.0);
        cc.update("energy", 98.0); // Below tolerance
        assert!(!cc.is_conserved("energy"));
    }

    #[test]
    fn test_conservation_increase_ok() {
        let mut cc = ConservationChecker::new();
        cc.register("energy", 100.0, 1.0);
        cc.update("energy", 110.0); // Increase is fine (one-sided)
        assert!(cc.is_conserved("energy"));
    }

    #[test]
    fn test_conservation_within_tolerance() {
        let mut cc = ConservationChecker::new();
        cc.register("energy", 100.0, 5.0);
        cc.update("energy", 96.0); // Within tolerance
        assert!(cc.is_conserved("energy"));
    }

    #[test]
    fn test_conservation_multiple_quantities() {
        let mut cc = ConservationChecker::new();
        cc.register("energy", 100.0, 1.0);
        cc.register("entropy", 50.0, 2.0);
        cc.update("energy", 99.0);
        cc.update("entropy", 47.0); // Below tolerance (50-2=48)
        let violations = cc.violations();
        assert!(violations.contains(&"entropy".to_string()));
        assert!(!violations.contains(&"energy".to_string()));
    }

    #[test]
    fn test_conservation_snapshot_history() {
        let mut cc = ConservationChecker::new();
        cc.register("energy", 100.0, 1.0);
        cc.snapshot(); // history[0]
        cc.update("energy", 95.0);
        cc.snapshot(); // history[1]
        assert_eq!(cc.history_len(), 2);
        assert_eq!(cc.history_value(0, "energy"), Some(100.0));
        assert_eq!(cc.history_value(1, "energy"), Some(95.0));
    }

    #[test]
    fn test_conservation_unknown_quantity() {
        let cc = ConservationChecker::new();
        assert!(cc.is_conserved("unknown")); // Unknown = trivially conserved
    }

    // === CracklePhase Tests ===

    #[test]
    fn test_crackle_basic() {
        let mut cp = CracklePhase::<i32>::new()
            .on_cool("no negatives", |vals| vals.iter().all(|v| *v >= 0));
        cp.fire(1);
        cp.fire(2);
        cp.fire(3);
        let result = cp.cool();
        assert!(result.is_sound());
        assert_eq!(result.total_values, 3);
    }

    #[test]
    fn test_crackle_violation() {
        let mut cp = CracklePhase::<i32>::new()
            .on_cool("all positive", |vals| vals.iter().all(|v| *v > 0));
        cp.fire(1);
        cp.fire(-1);
        cp.fire(3);
        let result = cp.cool();
        assert!(!result.is_sound());
        assert!(result.violations.contains(&"all positive".to_string()));
    }

    #[test]
    fn test_crackle_multiple_assertions() {
        let mut cp = CracklePhase::<i32>::new()
            .on_cool("all positive", |vals| vals.iter().all(|v| *v > 0))
            .on_cool("sum < 100", |vals| vals.iter().sum::<i32>() < 100)
            .on_cool("no duplicates", |vals| {
                let mut seen = std::collections::HashSet::new();
                vals.iter().all(|v| seen.insert(*v))
            });
        cp.fire(1);
        cp.fire(2);
        cp.fire(3);
        let result = cp.cool();
        assert!(result.is_sound());
        assert_eq!(result.assertion_count, 3);
    }

    #[test]
    fn test_crackle_cooling_detects_patterns() {
        let mut cp = CracklePhase::<f64>::new()
            .on_cool("variance reasonable", |vals| {
                if vals.is_empty() { return true; }
                let mean = vals.iter().sum::<f64>() / vals.len() as f64;
                let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64;
                var < 1000.0
            });
        for i in 0..100 {
            cp.fire((i as f64 * 0.1).sin());
        }
        let result = cp.cool();
        assert!(result.is_sound());
    }

    #[test]
    fn test_crackle_empty() {
        let mut cp = CracklePhase::<i32>::new()
            .on_cool("non-empty", |vals| !vals.is_empty());
        let result = cp.cool();
        assert!(!result.is_sound()); // empty violates "non-empty"
    }

    #[test]
    fn test_crackle_len() {
        let mut cp = CracklePhase::<i32>::new();
        assert!(cp.is_empty());
        cp.fire(1);
        assert_eq!(cp.len(), 1);
        assert!(!cp.is_empty());
    }

    // === CathedralProbe Tests ===

    #[test]
    fn test_cathedral_basic() {
        let mut cp = CathedralProbe::new(vec!["a", "b", "c"]);
        cp.connect("a", "b", 1.0);
        cp.connect("b", "c", 1.0);
        cp.connect("a", "c", 1.0);
        assert_eq!(cp.component_count(), 3);
        assert_eq!(cp.edge_count(), 3);
    }

    #[test]
    fn test_cathedral_spectrum() {
        let mut cp = CathedralProbe::new(vec!["a", "b", "c"]);
        cp.connect("a", "b", 1.0);
        cp.connect("b", "c", 1.0);
        cp.connect("a", "c", 1.0);
        let spectrum = cp.spectrum();
        assert_eq!(spectrum.len(), 3);
        // First eigenvalue of Laplacian should be ~0
        assert!(spectrum[0].abs() < 1.0);
    }

    #[test]
    fn test_cathedral_fiedler() {
        let mut cp = CathedralProbe::new(vec!["a", "b"]);
        cp.connect("a", "b", 1.0);
        let fiedler = cp.fiedler_value();
        assert!(fiedler > 0.0); // Connected graph has positive Fiedler value
    }

    #[test]
    fn test_cathedral_disconnected() {
        let cp = CathedralProbe::new(vec!["a", "b"]);
        // No edges — disconnected
        let fiedler = cp.fiedler_value();
        assert!(fiedler.abs() < 0.1); // Disconnected graph has ~0 Fiedler value
    }

    #[test]
    fn test_cathedral_healthy() {
        let mut cp = CathedralProbe::new(vec!["a", "b", "c"]);
        cp.connect("a", "b", 1.0);
        cp.connect("b", "c", 1.0);
        cp.connect("a", "c", 1.0);
        assert!(cp.is_healthy(0.1));
    }

    #[test]
    fn test_cathedral_cheeger() {
        let mut cp = CathedralProbe::new(vec!["a", "b"]);
        cp.connect("a", "b", 1.0);
        let cheeger = cp.cheeger_constant();
        assert!(cheeger >= 0.0 && cheeger <= 1.0);
    }

    #[test]
    fn test_cathedral_single_component() {
        let cp = CathedralProbe::new(vec!["a"]);
        let spectrum = cp.spectrum();
        assert_eq!(spectrum.len(), 1);
    }

    #[test]
    fn test_cathedral_empty() {
        let cp = CathedralProbe::new(vec![]);
        let spectrum = cp.spectrum();
        assert!(spectrum.is_empty());
    }

    #[test]
    fn test_cathedral_weighted_edges() {
        let mut cp = CathedralProbe::new(vec!["a", "b", "c"]);
        cp.connect("a", "b", 10.0); // Strong connection
        cp.connect("b", "c", 0.1);  // Weak connection
        let fiedler = cp.fiedler_value();
        assert!(fiedler > 0.0);
    }

    // === Integration Tests ===

    #[test]
    fn test_full_negative_space_workflow() {
        // Define a simple system and test its negative space
        let nt = NegativeTest::<Vec<i32>>::new()
            .forbid("empty output", |v| v.is_empty())
            .forbid("contains negative", |v| v.iter().any(|x| *x < 0))
            .forbid("unsorted", |v| {
                v.windows(2).any(|w| w[0] > w[1])
            });

        let output = vec![1, 2, 3, 4, 5];
        let result = nt.check(&output);
        assert!(result.is_clean());
    }

    #[test]
    fn test_conservation_with_crackle() {
        let mut cc = ConservationChecker::new();
        cc.register("total_output", 100.0, 1.0);

        let mut cp = CracklePhase::<f64>::new()
            .on_cool("sum near 100", |vals| {
                let total: f64 = vals.iter().sum();
                (total - 100.0).abs() < 5.0
            });

        // Fire 10 values that sum to 100
        for v in [10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0] {
            cp.fire(v);
        }
        cc.register("total_output", 100.0, 1.0);
        cc.update("total_output", 100.0);

        assert!(cc.is_conserved("total_output"));
        let crackle_result = cp.cool();
        assert!(crackle_result.is_sound());
    }

    #[test]
    fn test_spacemap_with_negative_test() {
        let mut sm = SpaceMap::<&str, Vec<i32>>::new();
        sm.forbid("error");
        sm.forbid("overflow");

        let nt = NegativeTest::<i32>::new()
            .forbid("negative", |x| *x < 0);

        let results = vec![("ok", vec![1, 2, 3]), ("error", vec![-1, 2, 3])];
        for (key, values) in &results {
            let result = nt.check_all(values);
            if result.is_clean() {
                sm.occupy(key, values.clone());
            }
        }

        // Only "ok" should be occupied
        assert_eq!(sm.occupied_count(), 1);
        assert!(sm.get(&"ok").is_some());
    }
}
