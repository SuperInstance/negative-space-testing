# negative-space-testing

**A Rust testing framework where you define what your system does NOT do.** Traditional tests check for expected outputs. This checks that forbidden outputs never appear — the pattern lives in the holes.

[![crates.io](https://img.shields.io/crates/v/negative-space-testing.svg)](https://crates.io/crates/negative-space-testing)
[![docs.rs](https://img.shields.rs/negative-space-testing/badge.svg)](https://docs.rs/negative-space-testing)

```toml
[dev-dependencies]
negative-space-testing = "0.1"
```

## 30-Second Example

```rust
use negative_space_testing::SpaceMap;

let mut space: SpaceMap<f64> = SpaceMap::new();
space.add_samples(predictions.iter().copied());

// Define forbidden zones — outputs that must never appear
space.exclude_fn("predictions below 0", |&p| p < 0.0);
space.exclude_fn("predictions above 1", |&p| p > 1.0);

let result = space.verify();
assert!(result.is_clean());               // no forbidden outputs detected
println!("openness: {:.0}%", result.openness() * 100.0);
```

## What It Does

Most testing frameworks ask: *does the system produce the right output?*  
This framework asks: *does the system avoid the wrong outputs?*

Five components, each testing a different kind of negative constraint:

| Component | What it tests |
|-----------|---------------|
| **SpaceMap** | Maps the output space of a system, marks forbidden regions, verifies nothing landed there |
| **NegativeTest** | Trait for defining per-output exclusion rules (like property-based testing, but negative) |
| **ConservationChecker** | Tracks quantities that must not regress (e.g., accuracy must not decrease, error rate must not increase) |
| **CracklePhase** | Defers assertions until after the "hot" computation phase — some failures are expected and honored |
| **CathedralProbe** | Tests the relationships *between* components, not the components themselves |

## Components

### SpaceMap — Output Landscape Mapping

Add actual outputs as samples, define forbidden regions, then verify nothing intruded. `openness()` tells you what fraction of samples fell in allowed space.

```rust
use negative_space_testing::SpaceMap;

let mut map: SpaceMap<f64> = SpaceMap::new();
map.add_samples(predictions.iter().copied());
map.exclude_fn("predictions below 0", |&p| p < 0.0);
map.exclude_fn("predictions above 1", |&p| p > 1.0);

let result = map.verify();
assert!(result.is_clean());
println!("openness: {:.0}%", result.openness() * 100.0);
```

### NegativeTest — Per-Output Exclusions

Define constraints on what code should **not** produce:

```rust
use negative_space_testing::NegativeTest;

struct NoPanic;

impl NegativeTest for NoPanic {
    type Output = Result<i32, String>;
    fn excludes(&self, v: &Result<i32, String>) -> bool { v.is_err() }
    fn description(&self) -> &str { "must not return an error" }
}
```

### ConservationChecker — Quantity Regression Detection

Track quantities that must not decrease (or increase, or drift from initial value). Useful for monitoring accuracy, coverage, error rates, entropy, or any metric that should be conserved across operations.

```rust
use negative_space_testing::ConservationChecker;

let mut checker = ConservationChecker::new();
checker.track_non_decreasing("test_coverage", 0.005); // 0.5% tolerance
checker.track_non_increasing("error_rate", 0.001);

for epoch in &training_results {
    checker.record("test_coverage", epoch.coverage);
    checker.record("error_rate", epoch.error_rate);
}

assert!(checker.check().is_conserved(), "a conserved quantity regressed");
```

Three conservation laws:
- `track_non_decreasing` — quantity must never decrease (e.g., accuracy, entropy)
- `track_non_increasing` — quantity must never increase (e.g., error rate)
- `track_conserved` — quantity must stay within tolerance of its initial value

### CracklePhase — Deferred Assertions

Assertions deferred during the "firing" phase (hot computation) are evaluated only after the system settles. Some cracks are failures. Some are expected — *kintsugi*, the crack honored rather than hidden.

```rust
use negative_space_testing::CracklePhase;

let mut phase = CracklePhase::new();

let final_loss = run_training();

phase.defer("loss below threshold", move || final_loss < 1.0);
phase.defer_crack("model not production-ready", move || final_loss < 0.01);

let result = phase.cool();
assert!(result.all_acceptable());
println!("beauty ratio: {:.0}%", result.beauty_ratio() * 100.0);
```

`CrackleOutcome` variants:
- `Smooth` — passed as expected
- `Craze` — failed unexpectedly (a defect)
- `Kintsugi` — failed as expected (beautiful, honored)
- `UnexpectedSmooth` — expected to fail but passed (interesting!)

### CathedralProbe — Inter-Component Relationship Testing

Tests whether the **relationship** between components has the right shape — not whether component A or B works individually, but whether they fit together correctly.

```rust
use negative_space_testing::CathedralProbe;

let mut probe = CathedralProbe::new();

probe.probe(
    "cache-database coherence",
    vec!["cache", "database"],
    "cache value must match authoritative database value",
    move || cache.get("user:42") == Some(&db.fetch("user:42")),
);

probe.probe(
    "serializer round-trip",
    vec!["serializer", "deserializer"],
    "deserialize(serialize(x)) == x",
    move || deserialize(&serialize(test_value)) == Some(test_value),
);

assert!(probe.verify().all_sound());
```

## Putting It Together

```rust
use negative_space_testing::{SpaceMap, ConservationChecker, CracklePhase, CathedralProbe};

// 1. Map what the system must NOT produce
let mut space: SpaceMap<Response> = SpaceMap::new();
space.add_samples(responses);
space.exclude_fn("no 5xx errors", |r| r.status >= 500);
space.exclude_fn("no empty bodies", |r| r.body.is_empty());
assert!(space.verify().is_clean());

// 2. Check conserved quantities
let mut conservation = ConservationChecker::new();
conservation.track_non_decreasing("p99_latency_headroom", 1.0);
for &measurement in &latency_samples { conservation.record("p99_latency_headroom", measurement); }
assert!(conservation.check().is_conserved());

// 3. Defer post-deployment assertions
let mut phase = CracklePhase::new();
phase.defer("cache hit rate acceptable", move || cache_hit_rate > 0.8);
phase.defer_crack("cache not fully warm yet", move || cache_hit_rate > 0.99);
assert!(phase.cool().all_acceptable());

// 4. Verify inter-component relationships
let mut cathedral = CathedralProbe::new();
cathedral.probe("auth-session contract", vec!["auth", "session"], "valid auth produces valid session", move || {
    let token = auth.login("user", "pass");
    session.validate(&token)
});
assert!(cathedral.verify().all_sound());
```

## Real-World Use Cases

- **ML model validation** — Verify predictions stay within bounds, accuracy never regresses across training epochs, and model outputs avoid forbidden regions
- **API contract testing** — Ensure responses never contain forbidden fields (PII, internal IDs), status codes stay in allowed ranges, and cache/database coherence holds
- **Data pipeline quality** — Check that record counts are conserved across transforms, deprecated fields don't leak, and inter-stage contracts are sound
- **Security boundary enforcement** — Define which zones/paths/resources are off-limits, then verify nothing crosses

## Installation

```toml
[dev-dependencies]
negative-space-testing = "0.1"
```

## Background & Philosophy

The name comes from a simple observation: the child looks at a cumulus cloud and sees a dragon. The meteorologist looks at the same cloud and sees *cumulus mediocris* — and in naming it, loses the dragon. Over-specification fills the negative space where imagination (and bugs) live.

The Jacquard card encodes its pattern in holes — absences where the hook must fall through. The pattern lives in what is missing. This framework tests the missing.

The cathedral is not the stone. It is the space the stone makes room for.

Inspired by explorations in creative constraint from the [Ford Creative Wheel](https://github.com/SuperInstance/AI-Writings/tree/main/ford-creative-wheel) collection.

## License

MIT — see [LICENSE](LICENSE).
