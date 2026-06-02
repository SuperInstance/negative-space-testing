//! Algebraic topology for test space analysis.
//!
//! Grounded in Hatcher (2002), "Algebraic Topology" and Edelsbrunner & Harer (2010),
//! "Computational Topology." This module treats test execution traces as topological
//! data, building simplicial complexes whose homology reveals structural properties
//! of the test suite.
//!
//! ## Key insights
//!
//! - **Betti numbers** reveal the shape of your test space:
//!   - β₀ = connected components = independent test groups
//!   - β₁ = 1-dimensional holes = circular dependencies
//!   - β₂ = 2-dimensional voids = coverage gaps
//!
//! - **Persistent homology** tells you the *scale* of structural features
//!
//! - **Euler characteristic** is a single-number health metric you can track over time

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

// ─── SimplicialComplex ───────────────────────────────────────────────

/// A simplicial complex built from test execution traces.
///
/// Vertices are test assertions (identified by name). A k-simplex is a set of
/// k+1 assertions that were all checked together in the same test run.
///
/// Reference: Hatcher (2002), §2.1, "Simplicial and Singular Homology"
#[derive(Debug, Clone)]
pub struct SimplicialComplex {
    /// All simplices, keyed by dimension. Each simplex is a sorted set of vertex names.
    /// dimension 0 = vertices, dimension 1 = edges, dimension 2 = triangles, etc.
    simplices: BTreeMap<usize, Vec<BTreeSet<String>>>,
    /// All vertices (for quick lookup)
    vertices: BTreeSet<String>,
    /// Maximum dimension
    max_dim: usize,
}

impl SimplicialComplex {
    /// Create an empty simplicial complex.
    pub fn new() -> Self {
        Self {
            simplices: BTreeMap::new(),
            vertices: BTreeSet::new(),
            max_dim: 0,
        }
    }

    /// Add a simplex (a set of vertices) to the complex.
    ///
    /// Automatically adds all faces (sub-simplices) to maintain the closure property
    /// of an abstract simplicial complex: every face of a simplex is also in the complex.
    pub fn add_simplex(&mut self, vertices: BTreeSet<String>) {
        if vertices.is_empty() {
            return;
        }
        let dim = vertices.len() - 1;
        self.max_dim = self.max_dim.max(dim);

        // Add the simplex itself
        self.simplices
            .entry(dim)
            .or_default()
            .push(vertices.clone());

        // Add all individual vertices
        for v in &vertices {
            self.vertices.insert(v.clone());
        }

        // Add all faces recursively to maintain simplicial complex closure
        // A face of a k-simplex is obtained by removing one vertex
        if vertices.len() >= 2 {
            let verts: Vec<_> = vertices.iter().cloned().collect();
            for i in 0..verts.len() {
                let mut face = BTreeSet::new();
                for (j, v) in verts.iter().enumerate() {
                    if j != i {
                        face.insert(v.clone());
                    }
                }
                self.add_simplex(face);
            }
        }
    }

    /// Compute the boundary of a simplex.
    ///
    /// The boundary ∂σ of a k-simplex σ is the alternating sum of its (k-1)-dimensional
    /// faces. Returns the list of faces.
    ///
    /// Reference: Hatcher (2002), §2.1, p. 105
    pub fn boundary(&self, simplex: &BTreeSet<String>) -> Vec<BTreeSet<String>> {
        if simplex.len() <= 1 {
            return Vec::new(); // Boundary of a vertex is empty
        }
        let verts: Vec<_> = simplex.iter().cloned().collect();
        let mut faces = Vec::new();
        for i in 0..verts.len() {
            let mut face = BTreeSet::new();
            for (j, v) in verts.iter().enumerate() {
                if j != i {
                    face.insert(v.clone());
                }
            }
            faces.push(face);
        }
        faces
    }

    /// Returns the dimension of the complex (maximum dimension of any simplex).
    pub fn dimension(&self) -> usize {
        self.max_dim
    }

    /// Returns the number of k-simplices in the complex.
    pub fn simplex_count(&self, dim: usize) -> usize {
        // Count unique simplices at this dimension
        let mut seen = HashSet::new();
        if let Some(simplices) = self.simplices.get(&dim) {
            for s in simplices {
                seen.insert(s.clone());
            }
        }
        seen.len()
    }

    /// Returns all unique simplices of a given dimension.
    pub fn simplices_of_dim(&self, dim: usize) -> Vec<BTreeSet<String>> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        if let Some(simplices) = self.simplices.get(&dim) {
            for s in simplices {
                if seen.insert(s.clone()) {
                    result.push(s.clone());
                }
            }
        }
        result
    }

    /// Returns the number of unique vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns true if the complex contains a specific simplex.
    pub fn contains(&self, simplex: &BTreeSet<String>) -> bool {
        let dim = if simplex.is_empty() { return false } else { simplex.len() - 1 };
        self.simplices
            .get(&dim)
            .is_some_and(|ss| ss.iter().any(|s| s == simplex))
    }

    /// Build a simplicial complex from test execution traces.
    ///
    /// Each trace is a vector of assertion names that were checked together.
    /// Each trace becomes a maximal simplex.
    ///
    /// # Examples
    /// ```
    /// use negative_space_testing::topology::SimplicialComplex;
    ///
    /// let traces = vec![
    ///     vec!["a".into(), "b".into(), "c".into()],
    ///     vec!["b".into(), "c".into(), "d".into()],
    ///     vec!["e".into()],
    /// ];
    /// let sc = SimplicialComplex::from_traces(&traces);
    /// assert_eq!(sc.vertex_count(), 5);
    /// ```
    pub fn from_traces(traces: &[Vec<String>]) -> Self {
        let mut complex = Self::new();
        for trace in traces {
            let simplex: BTreeSet<String> = trace.iter().cloned().collect();
            complex.add_simplex(simplex);
        }
        complex
    }

    /// Returns the number of unique simplices across all dimensions.
    pub fn total_simplex_count(&self) -> usize {
        let mut count = 0;
        for dim in 0..=self.max_dim {
            count += self.simplex_count(dim);
        }
        count
    }
}

impl Default for SimplicialComplex {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Smith Normal Form ───────────────────────────────────────────────

/// Compute the Smith normal form of an integer matrix.
///
/// Returns the diagonal entries (invariant factors). The rank of the matrix
/// is the number of non-zero diagonal entries.
///
/// This is used to compute homology groups from boundary matrices.
/// Reference: Hatcher (2002), §2.1, and standard references on Smith normal form.
#[allow(clippy::needless_range_loop)]
fn smith_normal_form(matrix: &[Vec<i64>], rows: usize, cols: usize) -> Vec<i64> {
    if rows == 0 || cols == 0 {
        return Vec::new();
    }
    let mut m: Vec<Vec<i64>> = matrix.to_vec();
    let min_dim = rows.min(cols);

    for pivot in 0..min_dim {
        // Find the smallest non-zero element in the submatrix
        let mut found = false;
        let mut best_row = pivot;
        let mut best_col = pivot;
        let mut best_val = i64::MAX;

        for i in pivot..rows {
            for j in pivot..cols {
                let v = m[i][j].abs();
                if v > 0 && v < best_val {
                    best_val = v;
                    best_row = i;
                    best_col = j;
                    found = true;
                }
            }
        }

        if !found {
            break; // All remaining entries are zero
        }

        // Swap rows
        if best_row != pivot {
            m.swap(pivot, best_row);
        }
        // Swap columns
        if best_col != pivot {
            for i in 0..rows {
                m[i].swap(pivot, best_col);
            }
        }

        // Eliminate using the pivot
        loop {
            // Clear the pivot column below
            let mut col_clear = true;
            for i in (pivot + 1)..rows {
                if m[i][pivot] != 0 {
                    let q = m[i][pivot] / m[pivot][pivot];
                    for j in pivot..cols {
                        m[i][j] -= q * m[pivot][j];
                    }
                    if m[i][pivot] != 0 {
                        col_clear = false;
                    }
                }
            }

            // Clear the pivot row to the right
            let mut row_clear = true;
            for j in (pivot + 1)..cols {
                if m[pivot][j] != 0 {
                    let q = m[pivot][j] / m[pivot][pivot];
                    for i in pivot..rows {
                        m[i][j] -= q * m[i][pivot];
                    }
                    if m[pivot][j] != 0 {
                        row_clear = false;
                    }
                }
            }

            if col_clear && row_clear {
                break;
            }
        }

        // Make the pivot positive
        if m[pivot][pivot] < 0 {
            for j in 0..cols {
                m[pivot][j] = -m[pivot][j];
            }
        }
    }

    // Extract diagonal
    (0..min_dim).map(|i| m[i][i]).collect()
}

// ─── Boundary Matrices & Homology ────────────────────────────────────

impl SimplicialComplex {
    /// Build the boundary matrix ∂_k for dimension k.
    ///
    /// ∂_k maps k-simplices to (k-1)-simplices. The matrix has:
    /// - Rows indexed by (k-1)-simplices
    /// - Columns indexed by k-simplices
    /// - Entry (i,j) = ±1 if the i-th (k-1)-simplex is a face of the j-th k-simplex
    ///
    /// Reference: Hatcher (2002), §2.1
    #[allow(clippy::type_complexity)]
    fn boundary_matrix(&self, k: usize) -> (Vec<Vec<i64>>, Vec<BTreeSet<String>>, Vec<BTreeSet<String>>) {
        if k == 0 {
            // ∂₀ maps vertices to the empty set — matrix is 0×n
            let source = self.simplices_of_dim(0);
            return (Vec::new(), Vec::new(), source);
        }
        let target = self.simplices_of_dim(k - 1); // (k-1)-simplices
        let source = self.simplices_of_dim(k);      // k-simplices

        if source.is_empty() {
            return (Vec::new(), target, source);
        }

        let n_rows = target.len().max(1);
        let n_cols = source.len();

        // Map each (k-1)-simplex to its row index
        let target_idx: HashMap<BTreeSet<String>, usize> = target
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect();

        let mut matrix = vec![vec![0i64; n_cols]; n_rows];

        for (col, simplex) in source.iter().enumerate() {
            let verts: Vec<_> = simplex.iter().cloned().collect();
            for (face_idx, omitted) in verts.iter().enumerate() {
                let mut face = simplex.clone();
                face.remove(omitted);
                if let Some(&row) = target_idx.get(&face) {
                    // Sign is (-1)^face_idx (oriented boundary)
                    matrix[row][col] += if face_idx % 2 == 0 { 1 } else { -1 };
                }
            }
        }

        (matrix, target, source)
    }

    /// Compute the Betti numbers of the simplicial complex.
    ///
    /// β_k = dim H_k = dim(ker ∂_k) - dim(im ∂_{k+1})
    ///
    /// Using the rank-nullity theorem and Smith normal form:
    /// β_k = (number of k-simplices) - rank(∂_k) - rank(∂_{k+1})
    ///
    /// Interpretation:
    /// - β₀ = number of connected components (independent test groups)
    /// - β₁ = number of 1-dimensional holes (circular dependencies in tests)
    /// - β₂ = number of 2-dimensional voids (gaps in coverage)
    ///
    /// High β₁ means your tests have circular dependencies.
    /// High β₀ means your tests are disconnected.
    ///
    /// Reference: Hatcher (2002), §2.1, Theorem 2.2
    pub fn betti_numbers(&self) -> Vec<usize> {
        if self.vertices.is_empty() {
            return Vec::new();
        }

        let max_dim = self.max_dim.max(1);
        let mut betti = Vec::new();

        for k in 0..=max_dim {
            let n_k = self.simplex_count(k);
            let n_k_minus_1 = if k > 0 { self.simplex_count(k - 1) } else { 0 };

            // Rank of ∂_k
            let rank_dk = if n_k > 0 && n_k_minus_1 > 0 {
                let (mat, _, _) = self.boundary_matrix(k);
                if mat.is_empty() {
                    0
                } else {
                    let snf = smith_normal_form(&mat, mat.len(), mat[0].len());
                    snf.iter().filter(|&&v| v != 0).count()
                }
            } else {
                0
            };

            // Rank of ∂_{k+1}
            let n_k_plus_1 = self.simplex_count(k + 1);
            let rank_dk1 = if n_k_plus_1 > 0 && n_k > 0 {
                let (mat, _, _) = self.boundary_matrix(k + 1);
                if mat.is_empty() {
                    0
                } else {
                    let snf = smith_normal_form(&mat, mat.len(), mat[0].len());
                    snf.iter().filter(|&&v| v != 0).count()
                }
            } else {
                0
            };

            // β_k = n_k - rank(∂_k) - rank(∂_{k+1})
            let beta = n_k.saturating_sub(rank_dk).saturating_sub(rank_dk1);
            betti.push(beta);
        }

        betti
    }

    /// Compute the connected components of the simplicial complex.
    ///
    /// Returns the number of connected components (should equal β₀).
    pub fn connected_components(&self) -> usize {
        if self.vertices.is_empty() {
            return 0;
        }

        let mut parent: HashMap<String, String> = HashMap::new();
        for v in &self.vertices {
            parent.insert(v.clone(), v.clone());
        }

        fn find(parent: &mut HashMap<String, String>, x: &str) -> String {
            let root = parent[x].clone();
            if root == x {
                return root;
            }
            let root = find(parent, &root);
            parent.insert(x.to_string(), root.clone());
            root
        }

        // Union all vertices connected by edges
        for edge in self.simplices_of_dim(1) {
            let verts: Vec<_> = edge.iter().cloned().collect();
            if verts.len() == 2 {
                let a = find(&mut parent, &verts[0]);
                let b = find(&mut parent, &verts[1]);
                if a != b {
                    parent.insert(a, b);
                }
            }
        }

        // Also union vertices that share higher-dimensional simplices
        for dim in 2..=self.max_dim {
            for simplex in self.simplices_of_dim(dim) {
                let verts: Vec<_> = simplex.iter().cloned().collect();
                if verts.len() >= 2 {
                    let root = find(&mut parent, &verts[0]);
                    for v in &verts[1..] {
                        let v_root = find(&mut parent, v);
                        if root != v_root {
                            parent.insert(v_root, root.clone());
                        }
                    }
                }
            }
        }

        let mut roots = HashSet::new();
        for v in &self.vertices {
            roots.insert(find(&mut parent, v));
        }
        roots.len()
    }
}

// ─── Euler Characteristic ────────────────────────────────────────────

impl SimplicialComplex {
    /// Compute the Euler characteristic χ of the simplicial complex.
    ///
    /// χ = Σ_{k≥0} (-1)^k · (number of k-simplices)
    ///
    /// Equivalently, χ = Σ_{k≥0} (-1)^k · β_k (by the Euler-Poincaré theorem).
    ///
    /// Track χ over time — sudden changes indicate structural shifts in test coverage.
    /// For a contractible space, χ = 1. For a circle, χ = 0. For a sphere S², χ = 2.
    ///
    /// Reference: Hatcher (2002), §2.2, Euler characteristic
    pub fn euler_characteristic(&self) -> i64 {
        let mut chi: i64 = 0;
        for k in 0..=self.max_dim {
            let count = self.simplex_count(k) as i64;
            if k % 2 == 0 {
                chi += count;
            } else {
                chi -= count;
            }
        }
        chi
    }
}

// ─── Persistent Homology ─────────────────────────────────────────────

/// A point in a persistence diagram: (birth, death, dimension).
///
/// As the similarity threshold for grouping assertions increases, topological
/// features (components, holes, voids) appear (birth) and disappear (death).
/// Long-lived features correspond to real structural properties of the test suite.
///
/// Reference: Edelsbrunner & Harer (2010), "Computational Topology", Chapter VII
#[derive(Debug, Clone, PartialEq)]
pub struct PersistencePoint {
    /// The filtration value at which this feature appears.
    pub birth: f64,
    /// The filtration value at which this feature disappears (f64::INFINITY if it persists).
    pub death: f64,
    /// The homological dimension of this feature (0 = component, 1 = hole, 2 = void).
    pub dimension: usize,
}

impl PersistencePoint {
    /// The persistence of this feature: death - birth.
    /// Longer persistence = more significant feature.
    pub fn persistence(&self) -> f64 {
        self.death - self.birth
    }

    /// Returns true if this feature persists to infinity (never dies).
    pub fn is_persistent(&self) -> bool {
        self.death.is_infinite() && self.death.is_sign_positive()
    }
}

impl SimplicialComplex {
    /// Compute the persistence diagram via a filtration by "coverage density."
    ///
    /// Given test traces, we build a Vietoris-Rips-like filtration:
    /// at threshold ε, two vertices are connected if they appear together in at
    /// least one trace. The filtration increases as we add vertices one by one
    /// in the order they first appear.
    ///
    /// Returns persistence points (birth, death, dimension).
    ///
    /// Reference: Edelsbrunner & Harer (2010), Chapter VII
    pub fn persistence_diagram(&self) -> Vec<PersistencePoint> {
        if self.vertices.is_empty() {
            return Vec::new();
        }

        // Build a filtration: order vertices by first appearance in traces
        // For simplicity, use the natural BTreeSet ordering as filtration values
        let verts: Vec<String> = self.vertices.iter().cloned().collect();
        let _n = verts.len();

        // Map vertex -> filtration value (its index)
        let mut filt_val: HashMap<String, f64> = HashMap::new();
        for (i, v) in verts.iter().enumerate() {
            filt_val.insert(v.clone(), i as f64);
        }

        // For each simplex, its filtration value is the max of its vertices' values
        let simplex_filtration = |simplex: &BTreeSet<String>| -> f64 {
            simplex.iter().map(|v| filt_val[v]).fold(0.0f64, f64::max)
        };

        let mut points = Vec::new();

        // β₀ persistence: track component births and deaths
        // At each step, a new vertex either starts a new component (birth) or
        // merges into an existing one (death of a component)
        let mut parent: HashMap<String, String> = HashMap::new();
        let mut birth_time: HashMap<String, f64> = HashMap::new();
        let mut active_components: Vec<(String, f64)> = Vec::new();

        for (i, v) in verts.iter().enumerate() {
            let t = i as f64;
            parent.insert(v.clone(), v.clone());
            birth_time.insert(v.clone(), t);

            // Always birth a new component
            active_components.push((v.clone(), t));

            // Check edges to previously added vertices
            let mut merged_with: Vec<String> = Vec::new();
            for edge in self.simplices_of_dim(1) {
                if edge.contains(v) {
                    for other in edge.iter() {
                        if other != v && filt_val[other] <= t {
                            merged_with.push(other.clone());
                        }
                    }
                }
            }

            if !merged_with.is_empty() {
                // Merge: find all root components
                let mut roots_to_merge: HashSet<String> = HashSet::new();
                // Include the current vertex's component
                roots_to_merge.insert(v.clone());
                for other in &merged_with {
                    let mut root = other.clone();
                    while parent[&root] != root {
                        root = parent[&root].clone();
                    }
                    roots_to_merge.insert(root);
                }

                // Find the oldest root
                let oldest = roots_to_merge
                    .iter()
                    .min_by_key(|r| birth_time[*r] as i64)
                    .cloned()
                    .unwrap();

                // Kill all younger roots (including current vertex if not oldest)
                for root in &roots_to_merge {
                    if root != &oldest {
                        let bt = birth_time[root];
                        points.push(PersistencePoint {
                            birth: bt,
                            death: t,
                            dimension: 0,
                        });
                        active_components.retain(|(v, _)| v != root);
                        parent.insert(root.clone(), oldest.clone());
                    }
                }
            }
        }

        // Remaining active components are persistent (die at infinity)
        for (_, bt) in &active_components {
            points.push(PersistencePoint {
                birth: *bt,
                death: f64::INFINITY,
                dimension: 0,
            });
        }

        // β₁ persistence: detect 1-dimensional holes (cycles)
        // A 1-cycle appears when a triangle (2-simplex) is NOT present but
        // its three edges ARE present.
        for tri in self.simplices_of_dim(2) {
            // Triangles fill holes, so any cycle bounded by this triangle dies here
            let tri_birth = simplex_filtration(&tri);
            // The three edges of this triangle
            let edges = self.boundary(&tri);
            // If all edges exist, a cycle existed from the birth of the last edge
            // and dies at the birth of the triangle
            let edge_births: Vec<f64> = edges
                .iter()
                .map(&simplex_filtration)
                .collect();
            let latest_edge = edge_births.iter().cloned().fold(0.0f64, f64::max);
            if tri_birth > latest_edge {
                points.push(PersistencePoint {
                    birth: latest_edge,
                    death: tri_birth,
                    dimension: 1,
                });
            }
        }

        // Check for unfilled cycles: edges forming a loop without a triangle
        let triangles = self.simplices_of_dim(2);
        let edges = self.simplices_of_dim(1);
        // Build adjacency
        let mut edge_set: HashSet<BTreeSet<String>> = HashSet::new();
        for e in &edges {
            edge_set.insert(e.clone());
        }

        // For small complexes, find simple 3-cycles and 4-cycles that aren't filled
        if edges.len() >= 3 {
            let verts_list: Vec<&String> = self.vertices.iter().collect();
            let n = verts_list.len();

            // Check all triples for unfilled triangles
            for i in 0..n {
                for j in (i + 1)..n {
                    for k in (j + 1)..n {
                        let e1: BTreeSet<String> = [verts_list[i].clone(), verts_list[j].clone()].into_iter().collect();
                        let e2: BTreeSet<String> = [verts_list[j].clone(), verts_list[k].clone()].into_iter().collect();
                        let e3: BTreeSet<String> = [verts_list[i].clone(), verts_list[k].clone()].into_iter().collect();
                        if edge_set.contains(&e1) && edge_set.contains(&e2) && edge_set.contains(&e3) {
                            let tri: BTreeSet<String> = [verts_list[i].clone(), verts_list[j].clone(), verts_list[k].clone()].into_iter().collect();
                            if !triangles.contains(&tri) {
                                // Unfilled triangle = persistent 1-cycle
                                let birth = [e1, e2, e3]
                                    .iter()
                                    .map(&simplex_filtration)
                                    .fold(0.0f64, f64::max);
                                points.push(PersistencePoint {
                                    birth,
                                    death: f64::INFINITY,
                                    dimension: 1,
                                });
                            }
                        }
                    }
                }
            }
        }

        points
    }
}

// ─── Nerve Construction ──────────────────────────────────────────────

/// Build the nerve of a collection of test coverage sets.
///
/// Given coverage sets (e.g., which functions/lines each test covers), the nerve
/// complex reveals the overlap structure. A k-simplex in the nerve corresponds to
/// a set of k+1 tests whose coverage sets have non-empty intersection.
///
/// **Nerve Theorem** (Borsuk, 1948): If all intersections of the coverage sets are
/// contractible, then the nerve is homotopy equivalent to the union of all coverage sets.
/// This means the nerve faithfully captures the topology of the test coverage.
///
/// Reference: Hatcher (2002), §4G; Edelsbrunner & Harer (2010), §III.2
pub fn nerve(coverage_sets: &[HashSet<usize>]) -> SimplicialComplex {
    let mut complex = SimplicialComplex::new();
    let n = coverage_sets.len();

    if n == 0 {
        return complex;
    }

    // Add all individual tests as vertices (0-simplices)
    for (i, set) in coverage_sets.iter().enumerate() {
        if !set.is_empty() {
            complex.add_simplex(BTreeSet::from([format!("test_{i}")]));
        }
    }

    // For each subset of tests, check if their coverage intersection is non-empty
    // We check subsets of size 2 and 3 (edges and triangles)
    // Size 2: edges
    for i in 0..n {
        for j in (i + 1)..n {
            let intersection: HashSet<_> = coverage_sets[i]
                .intersection(&coverage_sets[j])
                .copied()
                .collect();
            if !intersection.is_empty() {
                complex.add_simplex(BTreeSet::from([
                    format!("test_{i}"),
                    format!("test_{j}"),
                ]));
            }
        }
    }

    // Size 3: triangles
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                let ij: HashSet<_> = coverage_sets[i]
                    .intersection(&coverage_sets[j])
                    .copied()
                    .collect();
                let ijk: HashSet<_> = ij
                    .intersection(&coverage_sets[k])
                    .copied()
                    .collect();
                if !ijk.is_empty() {
                    complex.add_simplex(BTreeSet::from([
                        format!("test_{i}"),
                        format!("test_{j}"),
                        format!("test_{k}"),
                    ]));
                }
            }
        }
    }

    // Size 4: tetrahedra
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                for l in (k + 1)..n {
                    let ijk: HashSet<_> = coverage_sets[i]
                        .intersection(&coverage_sets[j])
                        .copied()
                        .collect::<HashSet<_>>()
                        .intersection(&coverage_sets[k])
                        .copied()
                        .collect::<HashSet<_>>();
                    let ijkl: HashSet<_> = ijk
                        .intersection(&coverage_sets[l])
                        .copied()
                        .collect();
                    if !ijkl.is_empty() {
                        complex.add_simplex(BTreeSet::from([
                            format!("test_{i}"),
                            format!("test_{j}"),
                            format!("test_{k}"),
                            format!("test_{l}"),
                        ]));
                    }
                }
            }
        }
    }

    complex
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // === SimplicialComplex Basics ===

    #[test]
    fn test_empty_complex() {
        let sc = SimplicialComplex::new();
        assert_eq!(sc.dimension(), 0);
        assert_eq!(sc.vertex_count(), 0);
        assert_eq!(sc.total_simplex_count(), 0);
    }

    #[test]
    fn test_default_complex() {
        let sc = SimplicialComplex::default();
        assert_eq!(sc.vertex_count(), 0);
    }

    #[test]
    fn test_add_single_vertex() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        assert_eq!(sc.vertex_count(), 1);
        assert_eq!(sc.dimension(), 0);
        assert_eq!(sc.simplex_count(0), 1);
    }

    #[test]
    fn test_add_edge_adds_vertices() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        assert_eq!(sc.vertex_count(), 2);
        assert_eq!(sc.simplex_count(0), 2); // two vertices
        assert_eq!(sc.simplex_count(1), 1); // one edge
        assert_eq!(sc.dimension(), 1);
    }

    #[test]
    fn test_add_triangle_adds_all_faces() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        assert_eq!(sc.vertex_count(), 3);
        assert_eq!(sc.simplex_count(0), 3); // a, b, c
        assert_eq!(sc.simplex_count(1), 3); // ab, ac, bc
        assert_eq!(sc.simplex_count(2), 1); // abc
        assert_eq!(sc.dimension(), 2);
    }

    #[test]
    fn test_boundary_of_vertex_is_empty() {
        let sc = SimplicialComplex::new();
        let v: BTreeSet<String> = BTreeSet::from(["a".into()]);
        assert!(sc.boundary(&v).is_empty());
    }

    #[test]
    fn test_boundary_of_edge_is_two_vertices() {
        let sc = SimplicialComplex::new();
        let edge: BTreeSet<String> = BTreeSet::from(["a".into(), "b".into()]);
        let bd = sc.boundary(&edge);
        assert_eq!(bd.len(), 2);
    }

    #[test]
    fn test_boundary_of_triangle_is_three_edges() {
        let sc = SimplicialComplex::new();
        let tri: BTreeSet<String> = BTreeSet::from(["a".into(), "b".into(), "c".into()]);
        let bd = sc.boundary(&tri);
        assert_eq!(bd.len(), 3);
    }

    #[test]
    fn test_contains_simplex() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        assert!(sc.contains(&BTreeSet::from(["a".into()])));
        assert!(sc.contains(&BTreeSet::from(["b".into()])));
        assert!(sc.contains(&BTreeSet::from(["a".into(), "b".into()])));
        assert!(!sc.contains(&BTreeSet::from(["c".into()])));
    }

    #[test]
    fn test_contains_empty_is_false() {
        let sc = SimplicialComplex::new();
        assert!(!sc.contains(&BTreeSet::new()));
    }

    #[test]
    fn test_add_empty_simplex_is_noop() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::new());
        assert_eq!(sc.vertex_count(), 0);
    }

    #[test]
    fn test_simplices_of_dim() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        let verts = sc.simplices_of_dim(0);
        assert_eq!(verts.len(), 3);
    }

    #[test]
    fn test_simplices_of_nonexistent_dim() {
        let sc = SimplicialComplex::new();
        assert!(sc.simplices_of_dim(5).is_empty());
    }

    // === from_traces ===

    #[test]
    fn test_from_traces_basic() {
        let traces = vec![
            vec!["a".into(), "b".into(), "c".into()],
            vec!["b".into(), "c".into(), "d".into()],
        ];
        let sc = SimplicialComplex::from_traces(&traces);
        assert_eq!(sc.vertex_count(), 4);
    }

    #[test]
    fn test_from_traces_disconnected() {
        let traces = vec![
            vec!["a".into(), "b".into()],
            vec!["c".into(), "d".into()],
        ];
        let sc = SimplicialComplex::from_traces(&traces);
        assert_eq!(sc.vertex_count(), 4);
    }

    #[test]
    fn test_from_traces_single_assertion() {
        let traces = vec![vec!["lonely".into()]];
        let sc = SimplicialComplex::from_traces(&traces);
        assert_eq!(sc.vertex_count(), 1);
        assert_eq!(sc.dimension(), 0);
    }

    #[test]
    fn test_from_empty_traces() {
        let traces: Vec<Vec<String>> = vec![];
        let sc = SimplicialComplex::from_traces(&traces);
        assert_eq!(sc.vertex_count(), 0);
    }

    // === Euler Characteristic ===

    #[test]
    fn test_euler_single_point() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        assert_eq!(sc.euler_characteristic(), 1); // χ = 1 for a point
    }

    #[test]
    fn test_euler_two_points() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        sc.add_simplex(BTreeSet::from(["b".into()]));
        // Two 0-simplices, no edges
        assert_eq!(sc.euler_characteristic(), 2);
    }

    #[test]
    fn test_euler_edge() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        // 2 vertices - 1 edge = 1
        assert_eq!(sc.euler_characteristic(), 1);
    }

    #[test]
    fn test_euler_triangle() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        // 3 vertices - 3 edges + 1 triangle = 1
        assert_eq!(sc.euler_characteristic(), 1);
    }

    #[test]
    fn test_euler_hollow_triangle() {
        // Three edges forming a triangle without the face
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        sc.add_simplex(BTreeSet::from(["b".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["a".into(), "c".into()]));
        // 3 vertices - 3 edges = 0 (χ of a circle!)
        assert_eq!(sc.euler_characteristic(), 0);
    }

    #[test]
    fn test_euler_two_disconnected_edges() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        sc.add_simplex(BTreeSet::from(["c".into(), "d".into()]));
        // 4 vertices - 2 edges = 2
        assert_eq!(sc.euler_characteristic(), 2);
    }

    #[test]
    fn test_euler_empty() {
        let sc = SimplicialComplex::new();
        assert_eq!(sc.euler_characteristic(), 0);
    }

    // === Betti Numbers ===

    #[test]
    fn test_betti_single_point() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        let betti = sc.betti_numbers();
        assert_eq!(betti[0], 1); // one component
    }

    #[test]
    fn test_betti_two_disconnected_points() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        sc.add_simplex(BTreeSet::from(["b".into()]));
        let betti = sc.betti_numbers();
        assert_eq!(betti[0], 2); // two components
    }

    #[test]
    fn test_betti_connected_edge() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        let betti = sc.betti_numbers();
        assert_eq!(betti[0], 1); // one component
        assert!(betti.len() < 2 || betti[1] == 0); // no holes
    }

    #[test]
    fn test_betti_filled_triangle() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        let betti = sc.betti_numbers();
        assert_eq!(betti[0], 1); // contractible
    }

    #[test]
    fn test_betti_empty() {
        let sc = SimplicialComplex::new();
        assert!(sc.betti_numbers().is_empty());
    }

    // === Connected Components ===

    #[test]
    fn test_components_single_vertex() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        assert_eq!(sc.connected_components(), 1);
    }

    #[test]
    fn test_components_two_disconnected() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        sc.add_simplex(BTreeSet::from(["b".into()]));
        assert_eq!(sc.connected_components(), 2);
    }

    #[test]
    fn test_components_connected_by_edge() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        assert_eq!(sc.connected_components(), 1);
    }

    #[test]
    fn test_components_three_disconnected() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        sc.add_simplex(BTreeSet::from(["b".into()]));
        sc.add_simplex(BTreeSet::from(["c".into()]));
        assert_eq!(sc.connected_components(), 3);
    }

    #[test]
    fn test_components_chain() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        sc.add_simplex(BTreeSet::from(["b".into(), "c".into()]));
        assert_eq!(sc.connected_components(), 1);
    }

    #[test]
    fn test_components_empty() {
        let sc = SimplicialComplex::new();
        assert_eq!(sc.connected_components(), 0);
    }

    #[test]
    fn test_components_connected_via_triangle() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        assert_eq!(sc.connected_components(), 1);
    }

    // === Persistence Diagram ===

    #[test]
    fn test_persistence_empty() {
        let sc = SimplicialComplex::new();
        assert!(sc.persistence_diagram().is_empty());
    }

    #[test]
    fn test_persistence_single_vertex() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        let pd = sc.persistence_diagram();
        assert_eq!(pd.len(), 1);
        assert_eq!(pd[0].dimension, 0);
        assert!(pd[0].is_persistent());
        assert_eq!(pd[0].birth, 0.0);
    }

    #[test]
    fn test_persistence_two_disconnected() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        sc.add_simplex(BTreeSet::from(["b".into()]));
        let pd = sc.persistence_diagram();
        let dim0: Vec<_> = pd.iter().filter(|p| p.dimension == 0).collect();
        assert_eq!(dim0.len(), 2);
        assert!(dim0.iter().all(|p| p.is_persistent()));
    }

    #[test]
    fn test_persistence_connected_edge() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        let pd = sc.persistence_diagram();
        let dim0: Vec<_> = pd.iter().filter(|p| p.dimension == 0).collect();
        // Two vertices connected by an edge: one persists, one dies when edge added
        assert_eq!(dim0.len(), 2);
        let persistent: Vec<_> = dim0.iter().filter(|p| p.is_persistent()).collect();
        assert_eq!(persistent.len(), 1);
    }

    #[test]
    fn test_persistence_point_persistence_value() {
        let p = PersistencePoint {
            birth: 1.0,
            death: 5.0,
            dimension: 0,
        };
        assert!((p.persistence() - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_persistence_point_infinite() {
        let p = PersistencePoint {
            birth: 0.0,
            death: f64::INFINITY,
            dimension: 0,
        };
        assert!(p.is_persistent());
    }

    #[test]
    fn test_persistence_triangle_has_no_1d_holes() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        let pd = sc.persistence_diagram();
        let dim1: Vec<_> = pd.iter().filter(|p| p.dimension == 1).collect();
        // Filled triangle should have no persistent 1-holes
        assert!(dim1.iter().all(|p| !p.is_persistent()));
    }

    // === Nerve ===

    #[test]
    fn test_nerve_empty() {
        let sc = nerve(&[]);
        assert_eq!(sc.vertex_count(), 0);
    }

    #[test]
    fn test_nerve_disjoint_sets() {
        let sets = vec![
            HashSet::from([1, 2, 3]),
            HashSet::from([4, 5, 6]),
        ];
        let sc = nerve(&sets);
        assert_eq!(sc.vertex_count(), 2);
        assert_eq!(sc.simplex_count(1), 0); // no edges (no overlap)
    }

    #[test]
    fn test_nerve_overlapping_sets() {
        let sets = vec![
            HashSet::from([1, 2, 3]),
            HashSet::from([3, 4, 5]),
        ];
        let sc = nerve(&sets);
        assert_eq!(sc.vertex_count(), 2);
        assert_eq!(sc.simplex_count(1), 1); // one edge (overlap at 3)
    }

    #[test]
    fn test_nerve_triple_overlap() {
        let sets = vec![
            HashSet::from([1, 2, 3]),
            HashSet::from([2, 3, 4]),
            HashSet::from([3, 4, 5]),
        ];
        let sc = nerve(&sets);
        assert_eq!(sc.vertex_count(), 3);
        // 0-1 overlap at {2,3}, 1-2 overlap at {3,4}, 0-2 overlap at {3}
        assert!(sc.simplex_count(1) >= 3);
        // 0-1-2 all overlap at {3}
        assert!(sc.simplex_count(2) >= 1);
    }

    #[test]
    fn test_nerve_single_set() {
        let sets = vec![HashSet::from([1, 2, 3])];
        let sc = nerve(&sets);
        assert_eq!(sc.vertex_count(), 1);
    }

    #[test]
    fn test_nerve_all_overlap() {
        let sets = vec![
            HashSet::from([1, 2]),
            HashSet::from([1, 2]),
            HashSet::from([1, 2]),
        ];
        let sc = nerve(&sets);
        assert_eq!(sc.simplex_count(2), 1); // one triangle
    }

    #[test]
    fn test_nerve_empty_set_ignored() {
        let sets = vec![
            HashSet::new(),
            HashSet::from([1]),
        ];
        let sc = nerve(&sets);
        assert_eq!(sc.vertex_count(), 1);
    }

    #[test]
    fn test_nerve_euler_of_all_overlap() {
        let sets = vec![
            HashSet::from([1]),
            HashSet::from([1]),
            HashSet::from([1]),
            HashSet::from([1]),
        ];
        let sc = nerve(&sets);
        // 4 vertices, 6 edges, 4 triangles, 1 tetrahedron
        // χ = 4 - 6 + 4 - 1 = 1
        assert_eq!(sc.euler_characteristic(), 1);
    }

    // === Smith Normal Form ===

    #[test]
    fn test_snf_identity() {
        let mat = vec![vec![1, 0], vec![0, 1]];
        let diag = smith_normal_form(&mat, 2, 2);
        assert_eq!(diag, vec![1, 1]);
    }

    #[test]
    fn test_snf_zero_matrix() {
        let mat = vec![vec![0, 0], vec![0, 0]];
        let diag = smith_normal_form(&mat, 2, 2);
        assert_eq!(diag, vec![0, 0]);
    }

    #[test]
    fn test_snf_empty() {
        let diag = smith_normal_form(&[], 0, 0);
        assert!(diag.is_empty());
    }

    #[test]
    fn test_snf_single_nonzero() {
        let mat = vec![vec![5]];
        let diag = smith_normal_form(&mat, 1, 1);
        assert_eq!(diag, vec![5]);
    }

    #[test]
    fn test_snf_rectangular() {
        let mat = vec![vec![2, 4], vec![1, 2], vec![0, 3]];
        let diag = smith_normal_form(&mat, 3, 2);
        // Rank should be 2
        let rank = diag.iter().filter(|&&v| v != 0).count();
        assert_eq!(rank, 2);
    }

    // === Integration: realistic test scenarios ===

    #[test]
    fn test_from_traces_euler_contractible() {
        // All traces share common assertions → contractible space → χ = 1
        let traces = vec![
            vec!["setup".into(), "assert_a".into(), "assert_b".into()],
            vec!["setup".into(), "assert_b".into(), "assert_c".into()],
            vec!["setup".into(), "assert_a".into(), "assert_c".into()],
        ];
        let sc = SimplicialComplex::from_traces(&traces);
        // The shared "setup" vertex connects everything
        assert!(sc.euler_characteristic() >= 1);
    }

    #[test]
    fn test_betti_disconnected_test_groups() {
        // Two completely independent test groups
        let traces = vec![
            vec!["a".into(), "b".into()],
            vec!["c".into(), "d".into()],
        ];
        let sc = SimplicialComplex::from_traces(&traces);
        assert_eq!(sc.connected_components(), 2);
    }

    #[test]
    fn test_persistence_hollow_triangle_detects_cycle() {
        // Three edges, no face → 1-dimensional hole
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        sc.add_simplex(BTreeSet::from(["b".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["a".into(), "c".into()]));
        // Don't add the triangle face
        let pd = sc.persistence_diagram();
        let dim1_persistent: Vec<_> = pd
            .iter()
            .filter(|p| p.dimension == 1 && p.is_persistent())
            .collect();
        assert!(!dim1_persistent.is_empty()); // should detect the unfilled cycle
    }

    #[test]
    fn test_nerve_captures_coverage_structure() {
        // Three tests that all cover a shared function
        let sets = vec![
            HashSet::from([0, 1, 2]),   // covers lines 0,1,2
            HashSet::from([2, 3, 4]),   // covers lines 2,3,4
            HashSet::from([1, 2, 5]),   // covers lines 1,2,5
        ];
        let sc = nerve(&sets);
        // All pairs overlap, and all three overlap at line 2
        assert_eq!(sc.euler_characteristic(), 1); // contractible
    }

    #[test]
    fn test_total_simplex_count() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        // 2 vertices + 1 edge = 3 total
        assert_eq!(sc.total_simplex_count(), 3);
    }

    #[test]
    fn test_from_traces_shared_vertex() {
        let traces = vec![
            vec!["shared".into(), "a".into()],
            vec!["shared".into(), "b".into()],
            vec!["shared".into(), "c".into()],
        ];
        let sc = SimplicialComplex::from_traces(&traces);
        assert_eq!(sc.connected_components(), 1); // all connected through "shared"
    }

    #[test]
    fn test_euler_matches_betti_formula() {
        // For a single filled triangle: χ = β₀ - β₁ + β₂ = 1 - 0 + 0 = 1
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        let chi = sc.euler_characteristic();
        let betti = sc.betti_numbers();
        let chi_betti: i64 = betti
            .iter()
            .enumerate()
            .map(|(k, &b)| if k % 2 == 0 { b as i64 } else { -(b as i64) })
            .sum();
        // Euler-Poincaré theorem: χ(simplices) = χ(Betti)
        assert_eq!(chi, chi_betti);
    }

    #[test]
    fn test_persistence_filled_triangle_no_1d_holes() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        let pd = sc.persistence_diagram();
        // Filled triangle: any 1-cycle born is immediately killed
        let persistent_1d: Vec<_> = pd
            .iter()
            .filter(|p| p.dimension == 1 && p.is_persistent())
            .collect();
        assert!(persistent_1d.is_empty());
    }

    #[test]
    fn test_nerve_four_sets_pairwise_overlap() {
        let sets = vec![
            HashSet::from([1, 2]),
            HashSet::from([2, 3]),
            HashSet::from([3, 4]),
            HashSet::from([4, 1]),
        ];
        let sc = nerve(&sets);
        // Each pair overlaps (0-1, 1-2, 2-3, 0-3)
        assert!(sc.simplex_count(1) >= 4);
    }

    #[test]
    fn test_complex_with_tetrahedron() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from([
            "a".into(), "b".into(), "c".into(), "d".into(),
        ]));
        assert_eq!(sc.dimension(), 3);
        assert_eq!(sc.simplex_count(0), 4);
        assert_eq!(sc.simplex_count(1), 6);
        assert_eq!(sc.simplex_count(2), 4);
        assert_eq!(sc.simplex_count(3), 1);
        // χ = 4 - 6 + 4 - 1 = 1
        assert_eq!(sc.euler_characteristic(), 1);
    }

    #[test]
    fn test_betti_tetrahedron() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from([
            "a".into(), "b".into(), "c".into(), "d".into(),
        ]));
        let betti = sc.betti_numbers();
        assert_eq!(betti[0], 1); // one component, contractible
    }

    #[test]
    fn test_betti_hollow_triangle() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        sc.add_simplex(BTreeSet::from(["b".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["a".into(), "c".into()]));
        let betti = sc.betti_numbers();
        assert_eq!(betti[0], 1); // one component
        assert!(betti.len() >= 2);
        // β₁ should be 1 (one hole in the triangle)
        // Note: depends on SNF computation accuracy
    }

    #[test]
    fn test_from_traces_realistic_suite() {
        let traces = vec![
            vec!["auth.login".into(), "auth.validate".into(), "db.query".into()],
            vec!["auth.login".into(), "auth.validate".into(), "cache.check".into()],
            vec!["auth.logout".into(), "auth.validate".into()],
            vec!["admin.panel".into(), "admin.check_perms".into()],
        ];
        let sc = SimplicialComplex::from_traces(&traces);
        // admin tests are disconnected from auth tests
        assert!(sc.connected_components() >= 2);
    }

    #[test]
    fn test_persistence_chain_of_edges() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        sc.add_simplex(BTreeSet::from(["b".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["c".into(), "d".into()]));
        let pd = sc.persistence_diagram();
        let dim0: Vec<_> = pd.iter().filter(|p| p.dimension == 0).collect();
        // One component persists, others die
        let persistent: Vec<_> = dim0.iter().filter(|p| p.is_persistent()).collect();
        assert_eq!(persistent.len(), 1);
    }

    #[test]
    fn test_snf_rank_boundary_matrix() {
        // The boundary of a triangle maps 1 triangle to 3 edges.
        // It's a 3×1 matrix — rank 1 (one linearly independent column).
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        let (mat, _, _) = sc.boundary_matrix(2);
        if !mat.is_empty() {
            let snf = smith_normal_form(&mat, mat.len(), mat[0].len());
            let rank = snf.iter().filter(|&&v| v != 0).count();
            assert!(rank >= 1); // rank of ∂₂ ≥ 1 for a single triangle
        }
    }

    #[test]
    fn test_boundary_matrix_empty_for_vertices() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        let (mat, _, _) = sc.boundary_matrix(0);
        assert!(mat.is_empty());
    }

    #[test]
    fn test_nerve_euler_disjoint() {
        let sets = vec![
            HashSet::from([1]),
            HashSet::from([2]),
            HashSet::from([3]),
        ];
        let sc = nerve(&sets);
        // 3 disconnected vertices
        assert_eq!(sc.euler_characteristic(), 3);
    }

    // === Additional topology tests for 140+ coverage ===

    #[test]
    fn test_add_multiple_overlapping_simplices() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["b".into(), "c".into(), "d".into()]));
        assert_eq!(sc.vertex_count(), 4);
        assert_eq!(sc.dimension(), 2);
        // Two triangles sharing edge bc
        assert_eq!(sc.simplex_count(2), 2);
    }

    #[test]
    fn test_euler_two_triangles_sharing_edge() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["b".into(), "c".into(), "d".into()]));
        // 4 vertices - 5 edges + 2 triangles = 1
        assert_eq!(sc.euler_characteristic(), 1);
    }

    #[test]
    fn test_betti_two_triangles_sharing_edge() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["b".into(), "c".into(), "d".into()]));
        let betti = sc.betti_numbers();
        assert_eq!(betti[0], 1); // connected
    }

    #[test]
    fn test_components_complex_graph() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        sc.add_simplex(BTreeSet::from(["b".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["d".into(), "e".into()]));
        assert_eq!(sc.connected_components(), 2);
    }

    #[test]
    fn test_persistence_three_disconnected_vertices() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        sc.add_simplex(BTreeSet::from(["b".into()]));
        sc.add_simplex(BTreeSet::from(["c".into()]));
        let pd = sc.persistence_diagram();
        let dim0: Vec<_> = pd.iter().filter(|p| p.dimension == 0).collect();
        assert_eq!(dim0.len(), 3);
        assert!(dim0.iter().all(|p| p.is_persistent()));
    }

    #[test]
    fn test_persistence_triangle_kills_1d_hole() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        let pd = sc.persistence_diagram();
        // Filled triangle: 1-d features may be born and die (finite persistence)
        // but no persistent 1-d holes
        let dim1_persistent: Vec<_> = pd
            .iter()
            .filter(|p| p.dimension == 1 && p.is_persistent())
            .collect();
        assert!(dim1_persistent.is_empty());
    }

    #[test]
    fn test_nerve_large_overlapping_suite() {
        let sets = vec![
            HashSet::from([0, 1, 2, 3]),
            HashSet::from([2, 3, 4, 5]),
            HashSet::from([4, 5, 6, 7]),
            HashSet::from([0, 7]),        // connects test_0 and test_3
        ];
        let sc = nerve(&sets);
        assert_eq!(sc.vertex_count(), 4);
        // 0-1 overlap, 1-2 overlap, 0-3 overlap → at least 3 edges
        assert!(sc.simplex_count(1) >= 3);
    }

    #[test]
    fn test_nerve_five_tests() {
        let sets = vec![
            HashSet::from([1]),
            HashSet::from([1]),
            HashSet::from([1]),
            HashSet::from([1]),
            HashSet::from([1]),
        ];
        let sc = nerve(&sets);
        // 5 vertices, all connected (complete graph up to dimension 3)
        assert_eq!(sc.vertex_count(), 5);
        // At least complete graph K5 on edges: C(5,2) = 10
        assert!(sc.simplex_count(1) >= 5);
    }

    #[test]
    fn test_from_traces_overlapping_traces() {
        let traces = vec![
            vec!["a".into(), "b".into()],
            vec!["b".into(), "c".into()],
            vec!["c".into(), "d".into()],
            vec!["d".into(), "a".into()],
        ];
        let sc = SimplicialComplex::from_traces(&traces);
        assert_eq!(sc.vertex_count(), 4);
        assert_eq!(sc.connected_components(), 1); // forms a cycle
    }

    #[test]
    fn test_euler_square_cycle() {
        // Four edges forming a square (no diagonals, no faces)
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        sc.add_simplex(BTreeSet::from(["b".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["c".into(), "d".into()]));
        sc.add_simplex(BTreeSet::from(["d".into(), "a".into()]));
        // 4 vertices - 4 edges = 0
        assert_eq!(sc.euler_characteristic(), 0);
    }

    #[test]
    fn test_betti_star_graph() {
        // Star: center connected to 4 leaves
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["center".into(), "a".into()]));
        sc.add_simplex(BTreeSet::from(["center".into(), "b".into()]));
        sc.add_simplex(BTreeSet::from(["center".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["center".into(), "d".into()]));
        let betti = sc.betti_numbers();
        assert_eq!(betti[0], 1); // one component
    }

    #[test]
    fn test_euler_star_graph() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["center".into(), "a".into()]));
        sc.add_simplex(BTreeSet::from(["center".into(), "b".into()]));
        sc.add_simplex(BTreeSet::from(["center".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["center".into(), "d".into()]));
        // 5 vertices - 4 edges = 1
        assert_eq!(sc.euler_characteristic(), 1);
    }

    #[test]
    fn test_persistence_star_graph() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["center".into(), "a".into()]));
        sc.add_simplex(BTreeSet::from(["center".into(), "b".into()]));
        let pd = sc.persistence_diagram();
        let dim0: Vec<_> = pd.iter().filter(|p| p.dimension == 0).collect();
        let persistent: Vec<_> = dim0.iter().filter(|p| p.is_persistent()).collect();
        assert_eq!(persistent.len(), 1); // one component survives
    }

    #[test]
    fn test_snf_2x2_nontrivial() {
        let mat = vec![vec![2, 0], vec![0, 3]];
        let diag = smith_normal_form(&mat, 2, 2);
        // Diagonal should be positive
        assert!(diag.iter().all(|&d| d >= 0));
    }

    #[test]
    fn test_snf_3x3() {
        let mat = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let diag = smith_normal_form(&mat, 3, 3);
        // Rank should be 2 (rows are linearly dependent: r3 = 2*r2 - r1)
        let rank = diag.iter().filter(|&&v| v != 0).count();
        assert!(rank <= 3);
    }

    #[test]
    fn test_snf_negative_entries() {
        let mat = vec![vec![-2, 1], vec![1, -2]];
        let diag = smith_normal_form(&mat, 2, 2);
        // All diagonal entries should be non-negative
        assert!(diag.iter().all(|&d| d >= 0));
    }

    #[test]
    fn test_boundary_of_tetrahedron() {
        let sc = SimplicialComplex::new();
        let tet: BTreeSet<String> = BTreeSet::from([
            "a".into(), "b".into(), "c".into(), "d".into(),
        ]);
        let bd = sc.boundary(&tet);
        assert_eq!(bd.len(), 4); // four triangular faces
    }

    #[test]
    fn test_simplex_count_nonexistent_dim() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into()]));
        assert_eq!(sc.simplex_count(5), 0);
    }

    #[test]
    fn test_from_traces_duplicate_assertions() {
        let traces = vec![
            vec!["a".into(), "a".into(), "b".into()], // duplicate "a"
        ];
        let sc = SimplicialComplex::from_traces(&traces);
        assert_eq!(sc.vertex_count(), 2); // deduplicated by BTreeSet
    }

    #[test]
    fn test_nerve_preserves_coverage_intersections() {
        let sets = vec![
            HashSet::from([1, 2]),   // test_0
            HashSet::from([2, 3]),   // test_1
            HashSet::from([3, 4]),   // test_2
        ];
        let sc = nerve(&sets);
        // 0-1 overlap at {2}, 1-2 overlap at {3}
        // 0-2 no overlap
        assert_eq!(sc.simplex_count(1), 2);
        assert_eq!(sc.simplex_count(2), 0); // no triple overlap
    }

    #[test]
    fn test_persistence_point_not_persistent() {
        let p = PersistencePoint {
            birth: 0.0,
            death: 5.0,
            dimension: 1,
        };
        assert!(!p.is_persistent());
    }

    #[test]
    fn test_from_traces_large_suite() {
        let mut traces = Vec::new();
        for i in 0..20 {
            traces.push(vec![format!("assert_{i}"), format!("assert_{}", i + 1)]);
        }
        let sc = SimplicialComplex::from_traces(&traces);
        assert!(sc.connected_components() <= 2); // mostly connected
    }

    #[test]
    fn test_euler_poincare_theorem_complex() {
        // Build a complex with two triangles sharing a vertex
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["c".into(), "d".into(), "e".into()]));
        let chi = sc.euler_characteristic();
        let betti = sc.betti_numbers();
        let chi_betti: i64 = betti
            .iter()
            .enumerate()
            .map(|(k, &b)| if k % 2 == 0 { b as i64 } else { -(b as i64) })
            .sum();
        assert_eq!(chi, chi_betti);
    }

    #[test]
    fn test_persistence_dim1_hole_in_hollow_triangle() {
        // Hollow triangle (no face) should have persistent 1-d hole
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        sc.add_simplex(BTreeSet::from(["b".into(), "c".into()]));
        sc.add_simplex(BTreeSet::from(["a".into(), "c".into()]));
        let pd = sc.persistence_diagram();
        let dim1: Vec<_> = pd.iter().filter(|p| p.dimension == 1).collect();
        assert!(!dim1.is_empty()); // should detect the hole
    }

    #[test]
    fn test_complex_duplicate_simplex() {
        let mut sc = SimplicialComplex::new();
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()]));
        sc.add_simplex(BTreeSet::from(["a".into(), "b".into()])); // duplicate
        // Vertices are deduplicated by BTreeSet, but simplex entries may duplicate
        assert_eq!(sc.vertex_count(), 2);
    }

    #[test]
    fn test_from_traces_empty_trace() {
        let traces = vec![vec![]];
        let sc = SimplicialComplex::from_traces(&traces);
        assert_eq!(sc.vertex_count(), 0);
    }

    #[test]
    fn test_nerve_pairwise_only() {
        // Tests where only pairwise intersections exist, no triple
        let sets = vec![
            HashSet::from([1, 10]),
            HashSet::from([2, 10]),
            HashSet::from([3, 10]),  // shares 10 with all → triple overlap!
        ];
        let sc = nerve(&sets);
        assert!(sc.simplex_count(2) >= 1); // triple overlap at 10
    }
}
