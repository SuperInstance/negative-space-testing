# negative-space-testing

> *"The meteorologist knows the cloud names too quickly — over-specification kills imagination."*

A Rust testing framework where you define what your system **does NOT do**, and the framework verifies the negative space.

---

## Philosophy

The child looks at a cumulus cloud and sees a dragon. The meteorologist looks at the same cloud and sees *cumulus mediocris* — and in naming it, loses the dragon. Over-specification fills the negative space where imagination (and bugs) live.

Traditional testing asks: *does the system produce the right output?*  
Negative-space testing asks: *does the system avoid the wrong outputs?*

These are not the same question. The Jacquard card encodes its pattern in holes — absences where the hook must fall through. The pattern lives in what is missing. This framework tests the missing.

Five components, each embodying a different facet of this insight:

---

## Components

### `NegativeTest` — the Jacquard card

Define constraints on what code should **not** produce. Like the holes in the Jacquard card: specify the absences, and the positive space defines itself.

```rust
use negative_space_testing::NegativeTest;

struct NoPanic;

impl NegativeTest for NoPanic {
    type Output = Result<i32, String>;
    fn excludes(&self, v: &Result<i32, String>) -> bool { v.is_err() }
    fn description(&self) -> &str { "must not return an error" }
}
```

---

### `SpaceMap` — the output landscape

Maps the full output space of a system. Add samples, add exclusions, verify that nothing landed in the negative space. The remaining open space is where your system is free to be creative.

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

`SpaceResult::openness()` tells you what fraction of samples fell in the positive (allowed) space — a measure of how much creative room your system has.

---

### `ConservationChecker` — the grand unification

Five hundred glazes revealed they were, at base, one mineral. The conserved quantity beneath all variation. `ConservationChecker` watches for quantities that must not be lost: accuracy, entropy, attention, coverage.

```rust
use negative_space_testing::ConservationChecker;

let mut checker = ConservationChecker::new();
checker.track_non_decreasing("test_coverage", 0.005); // tolerance of 0.5%
checker.track_non_increasing("error_rate", 0.001);

for epoch in &training_results {
    checker.record("test_coverage", epoch.coverage);
    checker.record("error_rate", epoch.error_rate);
}

assert!(checker.check().is_conserved(), "a conserved quantity regressed");
```

Three conservation laws:
- `track_non_decreasing` — quantity must never decrease (e.g., entropy, accuracy)
- `track_non_increasing` — quantity must never increase (e.g., error rate)
- `track_conserved` — quantity must stay within tolerance of its initial value

---

### `CracklePhase` — the cooling

*The glaze does not crack in the heat. The crack comes in the cooling.*

Assertions deferred during the "firing" phase (hot computation) are evaluated only after the system settles. Some cracks are failures. Some are beautiful — *kintsugi*, the crack honored rather than hidden.

```rust
use negative_space_testing::CracklePhase;

let mut phase = CracklePhase::new();

// During the "firing" phase — defer assertions
let final_loss = run_training(); // expensive computation

phase.defer("loss below threshold", move || final_loss < 1.0);

// Expected to crack — we know this model isn't perfect yet
phase.defer_crack("model not production-ready", move || final_loss < 0.01);

// During the "cooling" phase — evaluate everything
let result = phase.cool();
assert!(result.all_acceptable());
println!("beauty ratio: {:.0}%", result.beauty_ratio() * 100.0);
```

`CrackleOutcome` variants:
- `Smooth` — passed as expected
- `Craze` — failed unexpectedly (a defect)
- `Kintsugi` — failed as expected (beautiful, honored)
- `UnexpectedSmooth` — expected to fail but passed (interesting!)

---

### `CathedralProbe` — the space between stones

*The cathedral is not the stone. It is the space the stone makes room for.*

A `CathedralProbe` does not test whether component A works or component B works. It tests whether the **relationship** between A and B has the right shape — whether the flying buttress resolves the lateral thrust.

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

---

## Putting it together

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

---

## Installation

```toml
[dev-dependencies]
negative-space-testing = "0.1"
```

---

## License

MIT — see [LICENSE](LICENSE).

---

*"The walls aren't the cage. They're the dance floor. The silence between notes isn't absence. It's where the music lives."*
