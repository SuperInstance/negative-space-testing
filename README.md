# negative-space-testing

**Test what your code doesn't do.**

A testing framework where you define forbidden behaviors — things your code must *never* produce — and the framework verifies the negative space. Traditional tests specify what code *should* do. This catches what slips through the cracks.

```toml
[dependencies]
negative-space-testing = "0.1"
```

## 30-Second Example

```rust
use negative_space_testing::NegativeTest;

let test = NegativeTest::<i32>::new()
    .forbid("negative output", |x| *x < 0)
    .forbid("overflow (>1000)", |x| *x > 1000);

let result = test.check(&42);
assert!(result.is_clean());

let bad = test.check(&-1);
assert!(!bad.is_clean()); // Caught!
```

## What It Does

| Module | What It Tests |
|--------|--------------|
| **NegativeTest** | Define behaviors your code must never produce |
| **SpaceMap** | Map the full output space, flag intrusions into forbidden zones |
| **ConservationChecker** | Track quantities that should never decrease (energy, budget, quota) |
| **CracklePhase** | Deferred assertions that check patterns after all values are collected |
| **CathedralProbe** | Spectral analysis of your component graph — is the *space between* components healthy? |

## Real-World Use Cases

### API Response Validation
```rust
use negative_space_testing::NegativeTest;

let api_test = NegativeTest::<Response>::new()
    .forbid("internal error exposed", |r| r.body.contains("stack trace"))
    .forbid("auth token in body", |r| r.body.contains("Bearer "))
    .forbid("missing content-type", |r| !r.headers.contains_key("content-type"));

for response in api_responses {
    let result = api_test.check(&response);
    if !result.is_clean() {
        eprintln!("FORBIDDEN: {:?}", result.violations);
    }
}
```

### Budget Tracking
```rust
use negative_space_testing::ConservationChecker;

let mut budget = ConservationChecker::new();
budget.register("monthly_spend", 1000.0, 1.0);  // $1000 budget, $1 tolerance

for transaction in transactions {
    budget.update("monthly_spend", budget.current["monthly_spend"] - transaction.amount);
    if !budget.is_conserved("monthly_spend") {
        alert("Budget exceeded!");
    }
}
```

### Microservice Topology Health
```rust
use negative_space_testing::CathedralProbe;

let mut probe = CathedralProbe::new(vec!["auth", "api", "db", "cache"]);
probe.connect("auth", "api", 1.0);
probe.connect("api", "db", 1.0);
probe.connect("api", "cache", 0.5);

let fiedler = probe.fiedler_value(); // Higher = better connected
println!("Topology health: {} ({})", fiedler, 
    if probe.is_healthy(0.1) { "HEALTHY" } else { "FRAGMENTED" });
```

## API Reference

### `NegativeTest<T>`

Define forbidden behaviors for type `T`.

```rust
let test = NegativeTest::<Vec<i32>>::new()
    .forbid("empty", |v| v.is_empty())
    .forbid("unsorted", |v| v.windows(2).any(|w| w[0] > w[1]));

// Check single value
let result = test.check(&vec![1, 2, 3]);
assert!(result.is_clean());
assert_eq!(result.violations, vec![]);

// Check multiple values
let batch = test.check_all(&[vec![], vec![1, 3, 2], vec![1, 2, 3]]);
assert_eq!(batch.total_checked, 3);
assert_eq!(batch.clean_count, 1);  // Only the sorted non-empty one
```

### `SpaceMap<K, V>`

Map the full output space with forbidden zones.

```rust
let mut map = SpaceMap::<&str, Data>::new();
map.forbid("admin_panel");    // Must never appear in output
map.forbid("debug_endpoint");
map.occupy("user_profile", data);
map.occupy("admin_panel", leaked);  // Oops!

assert_eq!(map.check_intrusions(), vec!["admin_panel"]);
assert!(map.negative_space_ratio() < 1.0);  // Compromised!
```

### `ConservationChecker`

Track quantities that must not decrease (one-sided conservation).

```rust
let mut checker = ConservationChecker::new();
checker.register("energy", 100.0, 0.5);
checker.register("token_budget", 10000.0, 10.0);

checker.update("energy", 99.8);       // Fine — within tolerance
checker.update("token_budget", 12000.0); // Fine — increase allowed

assert!(checker.is_conserved("energy"));
assert!(checker.violations().is_empty());

checker.snapshot();  // Record history for analysis
```

### `CracklePhase<T>`

Collect values during execution, then check patterns during a "cooling" phase.

```rust
let mut phase = CracklePhase::<f64>::new()
    .on_cool("no negative variance", |vals| {
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        vals.iter().all(|v| v >= &mean * 0.5)
    })
    .on_cool("reasonable spread", |vals| {
        vals.iter().sum::<f64>() < 10000.0
    });

for sample in sensor_data {
    phase.fire(sample);
}

let result = phase.cool();
if !result.is_sound() {
    eprintln!("Pattern violations: {:?}", result.violations);
}
```

### `CathedralProbe`

Spectral analysis of component connectivity (Laplacian eigenvalues).

```rust
let mut probe = CathedralProbe::new(vec!["web", "api", "db", "queue"]);
probe.connect("web", "api", 1.0);
probe.connect("api", "db", 1.0);
probe.connect("api", "queue", 0.5);

println!("Fiedler value: {}", probe.fiedler_value());
println!("Cheeger constant: {}", probe.cheeger_constant());
println!("Spectrum: {:?}", probe.spectrum());
```

## How It Works

1. **NegativeTest** registers forbidden predicates and checks values against them. Unlike property testing, you specify what's *wrong*, not what's *right*.

2. **SpaceMap** builds a two-zone map (occupied vs. forbidden) and detects intrusions in O(1) lookup time.

3. **ConservationChecker** uses one-sided conservation: increases are always allowed, only decreases beyond tolerance trigger violations. This matches real-world invariants (budget, energy, quota).

4. **CracklePhase** accumulates values during "firing" and runs assertions during "cooling" — enabling pattern checks that require seeing the full dataset.

5. **CathedralProbe** computes the graph Laplacian of your component topology and extracts eigenvalues (spectrum), Fiedler value (connectivity), and Cheeger constant (bottleneck detection).

## Philosophy

*"The meteorologist knows the name of a cloud too quickly — over-specification kills imagination."*

Traditional testing over-specifies: "given input X, output Y." This misses everything in the negative space — the behaviors you didn't think to test for. This crate inverts the lens: define what's forbidden, and anything not forbidden is allowed. The space between your tests IS the product.

Inspired by the [Ford Creative Wheel](https://github.com/SuperInstance/AI-Writings/tree/main/ford-creative-wheel) experiments in the SuperInstance ecosystem.

## License

MIT
