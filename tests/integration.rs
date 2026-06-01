use negative_space_testing::{
    CathedralProbe, ConservationChecker, CracklePhase, SpaceMap,
};

/// Integration test: a simulated ML training loop with all five components.
///
/// The system trains a model for several epochs. We verify:
/// - The output space excludes degenerate predictions (SpaceMap)
/// - Accuracy is non-decreasing (ConservationChecker)
/// - Post-training assertions are deferred until after training (CracklePhase)
/// - Relationships between loss and accuracy hold (CathedralProbe)
#[test]
fn test_training_loop_scenario() {
    // Simulated training results: (epoch, loss, accuracy, predictions)
    let epochs: Vec<(usize, f64, f64, Vec<f64>)> = vec![
        (1, 2.3, 0.50, vec![0.1, 0.9, 0.7, 0.2]),
        (2, 1.8, 0.65, vec![0.2, 0.85, 0.75, 0.15]),
        (3, 1.2, 0.78, vec![0.05, 0.92, 0.88, 0.08]),
        (4, 0.8, 0.85, vec![0.03, 0.95, 0.91, 0.04]),
    ];

    // 1. SpaceMap: predictions must be in [0, 1]
    let mut space_map: SpaceMap<f64> = SpaceMap::new();
    for (_, _, _, preds) in &epochs {
        space_map.add_samples(preds.iter().copied());
    }
    space_map.exclude_fn("predictions below 0", |&p| p < 0.0);
    space_map.exclude_fn("predictions above 1", |&p| p > 1.0);
    assert!(space_map.verify().is_clean());

    // 2. ConservationChecker: accuracy must not decrease
    let mut checker = ConservationChecker::new();
    checker.track_non_decreasing("accuracy", 0.01);
    checker.track_non_increasing("loss", 0.01);
    for (_, loss, acc, _) in &epochs {
        checker.record("accuracy", *acc);
        checker.record("loss", *loss);
    }
    assert!(checker.check().is_conserved());

    // 3. CracklePhase: post-training assertions
    let final_accuracy = 0.85;
    let final_loss = 0.8;
    let mut phase = CracklePhase::new();
    phase.defer("accuracy above threshold", move || final_accuracy > 0.8);
    phase.defer("loss below initial", move || final_loss < 2.3);
    phase.defer_crack("model is not perfect", move || final_accuracy >= 1.0);
    assert!(phase.cool().all_acceptable());

    // 4. CathedralProbe: loss and accuracy should be inversely correlated
    let losses: Vec<f64> = epochs.iter().map(|(_, l, _, _)| *l).collect();
    let accs: Vec<f64> = epochs.iter().map(|(_, _, a, _)| *a).collect();
    let mut probe = CathedralProbe::new();
    probe.probe(
        "loss-accuracy inverse relationship",
        vec!["loss", "accuracy"],
        "as loss decreases, accuracy must increase",
        move || {
            losses.windows(2).all(|w| w[1] <= w[0])
                && accs.windows(2).all(|w| w[1] >= w[0])
        },
    );
    assert!(probe.verify().all_sound());
}

#[test]
fn test_negative_space_reveals_output_pattern() {
    // Define what a URL shortener output should NOT look like
    let outputs = vec![
        "https://short.ly/abc123",
        "https://short.ly/xyz789",
        "https://short.ly/mno456",
    ];

    let mut map: SpaceMap<&str> = SpaceMap::new();
    map.add_samples(outputs.iter().copied());
    map.exclude_fn("no http (must be https)", |s| s.starts_with("http://"));
    map.exclude_fn("no empty outputs", |s| s.is_empty());
    map.exclude_fn("no overly long codes", |s| s.len() > 30);

    let result = map.verify();
    assert!(result.is_clean());
    assert_eq!(result.positive_space_size, 3);
}

#[test]
fn test_crackle_phase_deferred_until_computation_complete() {
    // Simulate an expensive computation whose result is validated after
    let mut phase = CracklePhase::new();

    // "fire" the computation
    let mut accumulator = 0_i32;
    for i in 0..100 {
        accumulator += i;
    }
    let result = accumulator; // 4950

    // defer assertions — they run in the "cooling" phase
    phase.defer("sum is positive", move || result > 0);
    phase.defer("sum matches formula n*(n-1)/2", move || result == 99 * 100 / 2);
    phase.defer_crack("sum is unreasonably large", move || result > 1_000_000);

    let crackle_result = phase.cool();
    assert!(crackle_result.all_acceptable());
    assert_eq!(crackle_result.smooth, 2);
    assert_eq!(crackle_result.kintsugi, 1);
}

#[test]
fn test_conservation_checker_catches_regression() {
    let mut checker = ConservationChecker::new();
    checker.track_non_decreasing("test_coverage", 0.0);

    // Simulating a CI pipeline where coverage must not regress
    for &cov in &[0.70, 0.72, 0.75, 0.74] {
        // 0.74 < 0.75: regression!
        checker.record("test_coverage", cov);
    }

    let result = checker.check();
    assert!(!result.is_conserved());
    assert_eq!(result.violations[0].quantity_name, "test_coverage");
}

#[test]
fn test_cathedral_probe_catches_interface_contract_violation() {
    // Simulated: serializer and deserializer must round-trip correctly
    let serialize = |n: i32| format!("{n}");
    let deserialize = |s: &str| s.parse::<i32>().ok();

    let test_value = 42_i32;
    let serialized = serialize(test_value);

    let mut probe = CathedralProbe::new();
    probe.probe(
        "serializer-deserializer round trip",
        vec!["serializer", "deserializer"],
        "deserialize(serialize(x)) == x for all x",
        move || deserialize(&serialized) == Some(test_value),
    );

    assert!(probe.verify().all_sound());
}

#[test]
fn test_combined_negative_and_conservation() {
    // A queue that should never be empty after enqueue, and size must be non-decreasing
    let mut sizes: Vec<usize> = Vec::new();
    let mut queue: Vec<i32> = Vec::new();

    for i in 0..5 {
        queue.push(i);
        sizes.push(queue.len());
    }

    let mut map: SpaceMap<usize> = SpaceMap::new();
    map.add_samples(sizes.iter().copied());
    map.exclude_fn("queue must not be empty after enqueue", |&s| s == 0);

    let mut checker = ConservationChecker::new();
    checker.track_non_decreasing("queue_size", 0.0);
    for &s in &sizes {
        checker.record("queue_size", s as f64);
    }

    assert!(map.verify().is_clean());
    assert!(checker.check().is_conserved());
}
